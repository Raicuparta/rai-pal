use std::{
	collections::{
		BTreeMap,
		HashMap,
	},
	fs,
	hash::BuildHasher,
	path::{
		Path,
		PathBuf,
	},
};

use steamlocate::SteamDir;

use crate::{
	game::DbGame,
	result::Result,
};

const RAI_PAL_USER_SETTINGS_START_MARKER: &str = "# >>> RAI PAL MANAGED USER SETTINGS START >>>";
const RAI_PAL_USER_SETTINGS_END_MARKER: &str = "# <<< RAI PAL MANAGED USER SETTINGS END <<<";

pub(crate) fn set_environment_for_game<S>(
	game: &DbGame,
	environment: &HashMap<String, String, S>,
) -> Result
where
	S: BuildHasher,
{
	if environment.is_empty() {
		return Ok(());
	}

	if let Some(user_settings_path) = get_proton_user_settings_path(game) {
		ensure_user_settings_exists(&user_settings_path)?;
		upsert_user_settings(&user_settings_path, environment)?;
	} else {
		log::warn!(
			"Failed to resolve Proton user_settings.py for Steam game `{}` ({})",
			game.display_title,
			game.external_id,
		);
	}

	Ok(())
}

fn get_app_id(game: &DbGame) -> Option<u32> {
	game.external_id.parse().ok()
}

fn get_app_library_path(steam_dir: &SteamDir, app_id: u32) -> Option<PathBuf> {
	for library in (steam_dir.libraries().ok()?).flatten() {
		if library.app_ids().contains(&app_id) {
			return Some(library.path().to_path_buf());
		}
	}

	None
}

fn get_compatdata_version(library_path: &Path, app_id: u32) -> Option<String> {
	let version_path = library_path
		.join("steamapps")
		.join("compatdata")
		.join(app_id.to_string())
		.join("version");

	fs::read_to_string(version_path)
		.ok()
		.map(|version| version.trim().to_string())
		.filter(|version| !version.is_empty())
}

fn get_configured_compat_tool_name(steam_dir: &SteamDir, app_id: u32) -> Option<String> {
	let compat_mapping = steam_dir.compat_tool_mapping().unwrap_or_else(|error| {
		log::error!("Failed to read Steam compatibility mapping: {error}");
		HashMap::new()
	});

	compat_mapping
		.get(&app_id)
		.and_then(|tool| tool.name.clone())
		.filter(|tool_name| !tool_name.is_empty())
		.or_else(|| {
			compat_mapping
				.get(&0)
				.and_then(|tool| tool.name.clone())
				.filter(|tool_name| !tool_name.is_empty())
		})
}

fn parse_proton_major(tool_name: &str) -> Option<String> {
	tool_name
		.strip_prefix("proton_")
		.filter(|major| !major.is_empty())
		.map(ToOwned::to_owned)
}

fn list_tool_directories(steam_dir: &SteamDir) -> Vec<PathBuf> {
	let mut roots = vec![steam_dir.path().join("compatibilitytools.d")];

	for library in (steam_dir.libraries().ok()).into_iter().flatten().flatten() {
		roots.push(library.path().join("steamapps").join("common"));
	}

	let mut tool_dirs = Vec::new();
	for root in roots {
		let Ok(entries) = fs::read_dir(&root) else {
			continue;
		};

		for entry in entries.flatten() {
			let path = entry.path();
			if path.is_dir() {
				tool_dirs.push(path);
			}
		}
	}

	tool_dirs
}

fn tool_manifest_mentions_name(tool_path: &Path, tool_name: &str) -> bool {
	let manifest_path = tool_path.join("compatibilitytool.vdf");
	let Ok(manifest_content) = fs::read_to_string(manifest_path) else {
		return false;
	};

	manifest_content
		.to_ascii_lowercase()
		.contains(&format!("\"{}\"", tool_name.to_ascii_lowercase()))
}

fn resolve_proton_path(
	steam_dir: &SteamDir,
	tool_name: Option<&str>,
	version_hint: Option<&str>,
) -> Option<PathBuf> {
	let mut tool_dirs = list_tool_directories(steam_dir);
	tool_dirs.sort();

	if let Some(tool_name) = tool_name {
		if let Some(path) = tool_dirs
			.iter()
			.find(|path| {
				path.file_name()
					.and_then(|name| name.to_str())
					.is_some_and(|name| {
						name.eq_ignore_ascii_case(tool_name)
							|| tool_manifest_mentions_name(path, tool_name)
					})
			})
			.cloned()
		{
			return Some(path);
		}

		if let Some(proton_major) = parse_proton_major(tool_name)
			&& let Some(path) = tool_dirs
				.iter()
				.find(|path| {
					path.file_name()
						.and_then(|name| name.to_str())
						.is_some_and(|name| {
							name.to_ascii_lowercase()
								.starts_with(&format!("proton {proton_major}").to_ascii_lowercase())
						})
				})
				.cloned()
		{
			return Some(path);
		}

		if tool_name.eq_ignore_ascii_case("proton_experimental")
			&& let Some(path) = tool_dirs
				.iter()
				.find(|path| {
					path.file_name()
						.and_then(|name| name.to_str())
						.is_some_and(|name| name.to_ascii_lowercase().contains("experimental"))
				})
				.cloned()
		{
			return Some(path);
		}
	}

	if let Some(version_hint) = version_hint {
		let version_prefix = version_hint.split('-').next().unwrap_or(version_hint);
		let version_candidates = [
			version_hint.to_ascii_lowercase(),
			version_prefix.to_ascii_lowercase(),
		];

		if let Some(path) = tool_dirs
			.iter()
			.find(|path| {
				path.file_name()
					.and_then(|name| name.to_str())
					.is_some_and(|name| {
						let name = name.to_ascii_lowercase();
						version_candidates
							.iter()
							.any(|candidate| name.contains(candidate))
					})
			})
			.cloned()
		{
			return Some(path);
		}
	}

	None
}

fn get_proton_user_settings_path(game: &DbGame) -> Option<PathBuf> {
	let app_id = get_app_id(game)?;
	let steam_dir = SteamDir::locate().ok()?;
	let library_path = get_app_library_path(&steam_dir, app_id)?;
	let version_hint = get_compatdata_version(&library_path, app_id);
	let configured_tool_name = get_configured_compat_tool_name(&steam_dir, app_id);

	let proton_path = resolve_proton_path(
		&steam_dir,
		configured_tool_name.as_deref(),
		version_hint.as_deref(),
	)?;

	Some(proton_path.join("user_settings.py"))
}

fn ensure_user_settings_exists(path: &Path) -> Result {
	if path.exists() {
		return Ok(());
	}

	if let Some(parent) = path.parent() {
		fs::create_dir_all(parent)?;
		fs::write(path, "")?;
	}

	Ok(())
}

fn find_managed_block_bounds(content: &str, marker: &str) -> Option<(usize, usize)> {
	let marker_index = content.find(marker)?;
	let line_start = content[..marker_index]
		.rfind('\n')
		.map_or(0, |index| index + 1);
	let line_end = content[marker_index..]
		.find('\n')
		.map_or(content.len(), |offset| marker_index + offset + 1);

	Some((line_start, line_end))
}

fn format_managed_user_settings_block<S>(
	environment: &HashMap<String, String, S>,
	newline: &str,
) -> Result<String>
where
	S: BuildHasher,
{
	let mut keys = environment.keys().cloned().collect::<Vec<_>>();
	keys.sort();

	let ordered_environment = keys
		.into_iter()
		.map(|key| {
			(
				key.clone(),
				environment.get(&key).cloned().unwrap_or_default(),
			)
		})
		.collect::<BTreeMap<_, _>>();

	let serialized_environment = serde_json::to_string(&ordered_environment)?;

	let managed_oneliner = format!(
		"user_settings = globals().get(\"user_settings\") if isinstance(globals().get(\"user_settings\"), dict) else {{}}; user_settings.update({serialized_environment})"
	);

	let lines = [
		RAI_PAL_USER_SETTINGS_START_MARKER.to_string(),
		managed_oneliner,
		RAI_PAL_USER_SETTINGS_END_MARKER.to_string(),
	];

	let mut block = lines.join(newline);
	block.push_str(newline);

	Ok(block)
}

fn upsert_user_settings<S>(path: &Path, environment: &HashMap<String, String, S>) -> Result
where
	S: BuildHasher,
{
	let existing = fs::read_to_string(path).unwrap_or_default();
	let newline = if existing.contains("\r\n") {
		"\r\n"
	} else {
		"\n"
	};
	let settings_block = format_managed_user_settings_block(environment, newline)?;

	let mut updated = if let (Some((start_line_start, _)), Some((_, end_line_end))) = (
		find_managed_block_bounds(&existing, RAI_PAL_USER_SETTINGS_START_MARKER),
		find_managed_block_bounds(&existing, RAI_PAL_USER_SETTINGS_END_MARKER),
	) {
		let mut updated = String::new();
		updated.push_str(&existing[..start_line_start]);
		updated.push_str(&settings_block);
		if end_line_end < existing.len() {
			updated.push_str(&existing[end_line_end..]);
		}
		updated
	} else {
		let mut updated = existing;
		if !updated.trim().is_empty() && !updated.ends_with(newline) {
			updated.push_str(newline);
		}
		if !updated.trim().is_empty() {
			updated.push_str(newline);
		}
		updated.push_str(&settings_block);
		updated
	};

	if !updated.is_empty() && !updated.ends_with(newline) {
		updated.push_str(newline);
	}

	fs::write(path, updated)?;

	Ok(())
}
