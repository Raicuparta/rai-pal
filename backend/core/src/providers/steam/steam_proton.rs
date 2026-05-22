use std::{
	collections::HashMap,
	fs,
	path::{
		Path,
		PathBuf,
	},
	process::Command,
};

use anyhow::bail;
use steamlocate::SteamDir;

use crate::{
	game::DbGame,
	paths,
	result::CoreResult,
};

const DLL_OVERRIDES_SECTION: &str = "[Software\\\\Wine\\\\DllOverrides]";
const DLL_OVERRIDE_VALUE: &str = "native,builtin";

pub fn get_wine_prefix_path(game: &DbGame) -> CoreResult<PathBuf> {
	let app_id = game.external_id.parse()?;
	let steam_dir = SteamDir::locate()?;

	let library_path = get_app_library_path(&steam_dir, app_id)?;

	Ok(library_path
		.join("steamapps")
		.join("compatdata")
		.join(app_id.to_string())
		.join("pfx"))
}

pub fn get_wine_binary_path(game: &DbGame) -> CoreResult<PathBuf> {
	let prefix_path = get_wine_prefix_path(game)?;
	let compat_data_path = paths::path_parent(&prefix_path)?;
	let config_info_path = compat_data_path.join("config_info");

	let config_info_data = fs::read_to_string(&config_info_path)?;

	let proton_lib_path_line = match config_info_data.lines().nth(2) {
		Some(line) if !line.trim().is_empty() => line.trim(),
		_ => {
			bail!(
				"Steam Proton config_info for `{}` ({}) is missing a valid third line at {}",
				game.display_title,
				game.external_id,
				config_info_path.display(),
			);
		}
	};

	let proton_lib_path = Path::new(proton_lib_path_line);

	Ok(paths::path_parent(proton_lib_path)?
		.join("bin")
		.join("wine"))
}

pub fn run_with_wine(
	game: &DbGame,
	exe_path: &Path,
	args: &[String],
	wine_env: &HashMap<String, String>,
) -> CoreResult {
	let wine_prefix_path = get_wine_prefix_path(game)?;

	let wine_binary_path = get_wine_binary_path(game)?;

	let compat_data_path = paths::path_parent(&wine_prefix_path)?;

	let child = Command::new(&wine_binary_path)
		.env("WINEPREFIX", &wine_prefix_path)
		.env("STEAM_COMPAT_DATA_PATH", compat_data_path)
		.env("WINEFSYNC", "1")
		.envs(wine_env)
		.arg(exe_path)
		.args(args)
		.spawn()?;

	log::info!(
		"Launched `{}` with Wine `{}` for Steam game `{}` ({}) using prefix `{}` (pid {})",
		exe_path.display(),
		wine_binary_path.display(),
		game.display_title,
		game.external_id,
		wine_prefix_path.display(),
		child.id(),
	);

	Ok(())
}

pub(crate) fn set_wine_dll_overrides_for_game(
	game: &DbGame,
	dll_overrides: &[String],
) -> CoreResult {
	if dll_overrides.is_empty() {
		log::debug!(
			"Steam Proton DLL override setup skipped for `{}` ({}): empty override list",
			game.display_title,
			game.external_id,
		);
		return Ok(());
	}

	log::info!(
		"Steam Proton DLL override setup started for `{}` ({}) with {} DLL entries",
		game.display_title,
		game.external_id,
		dll_overrides.len(),
	);

	let user_reg_path = get_wine_prefix_path(game)?.join("user.reg");
	upsert_dll_overrides_in_user_reg(&user_reg_path, dll_overrides)?;

	Ok(())
}

fn get_app_library_path(steam_dir: &SteamDir, app_id: u32) -> CoreResult<PathBuf> {
	for library_result in steam_dir.libraries()? {
		let library = match library_result {
			Ok(library) => library,
			Err(error) => {
				log::warn!("Failed to read a Steam library entry: {error}");
				continue;
			}
		};

		if library.app_ids().contains(&app_id) {
			log::debug!(
				"Found Steam app {} in library {}",
				app_id,
				library.path().display(),
			);
			return Ok(library.path().to_path_buf());
		}
	}

	bail!("Steam app {app_id} was not found in any Steam library");
}

fn upsert_dll_overrides_in_user_reg(path: &Path, dll_overrides: &[String]) -> CoreResult {
	if !path.exists() {
		bail!(
			"Steam Proton user.reg does not exist yet at {}. Launch the game once to create the prefix.",
			path.display(),
		);
	}

	let user_reg_data = fs::read_to_string(path)?;
	let mut ensured_user_reg_data = user_reg_data.clone();

	for dll_override in dll_overrides {
		let normalized_name = normalize_dll_override_name(dll_override);
		ensured_user_reg_data = reg_add_in_section(
			&ensured_user_reg_data,
			DLL_OVERRIDES_SECTION,
			&normalized_name,
			DLL_OVERRIDE_VALUE,
		);
	}

	if user_reg_data != ensured_user_reg_data {
		let backup_path = path.parent().map_or_else(
			|| path.with_extension("reg.bak"),
			|parent| parent.join("user.reg.bak"),
		);

		fs::copy(path, backup_path)?;
		fs::write(path, ensured_user_reg_data)?;

		log::info!("Updated Steam Proton user.reg at {}", path.display());
	}

	Ok(())
}

fn normalize_dll_override_name(dll_name: &str) -> String {
	if dll_name.len() > 4 && dll_name.to_ascii_lowercase().ends_with(".dll") {
		dll_name[..dll_name.len() - 4].to_string()
	} else {
		dll_name.to_string()
	}
}

fn reg_add_in_section(reg_data: &str, section: &str, key: &str, value: &str) -> String {
	let newline = if reg_data.contains("\r\n") {
		"\r\n"
	} else {
		"\n"
	};

	let mut lines = if reg_data.is_empty() {
		Vec::new()
	} else {
		reg_data
			.split(newline)
			.map(std::string::ToString::to_string)
			.collect::<Vec<_>>()
	};

	let key_prefix = format!("\"{key}\"=");
	let key_value_line = format!("\"{key}\"=\"{value}\"");

	if let Some(section_start) = lines
		.iter()
		.position(|line| line.trim_start().starts_with(section))
	{
		let section_end_candidate = lines
			.iter()
			.enumerate()
			.skip(section_start + 1)
			.find(|(_, line)| line.trim_start().starts_with('['));

		let section_end = if let Some((index, _)) = section_end_candidate {
			index
		} else {
			lines.len()
		};

		if let Some(existing_key_index) = lines
			.iter()
			.enumerate()
			.skip(section_start + 1)
			.take(section_end.saturating_sub(section_start + 1))
			.find(|(_, line)| line.trim_start().starts_with(&key_prefix))
			.map(|(index, _)| index)
		{
			lines[existing_key_index] = key_value_line;
		} else {
			lines.insert(section_end, key_value_line);
		}
	} else {
		if !lines.is_empty() && !lines.last().is_some_and(String::is_empty) {
			lines.push(String::new());
		}

		lines.push(section.to_string());
		lines.push(key_value_line);
	}

	let mut updated = lines.join(newline);
	if !updated.ends_with(newline) {
		updated.push_str(newline);
	}

	updated
}
