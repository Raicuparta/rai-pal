use std::{
	collections::HashMap,
	fs,
	path::{
		Path,
		PathBuf,
	},
	process::Command,
};

use anyhow::Context;
use steamlocate::SteamDir;

use crate::{
	game::DbGame,
	result::Result,
};

const DLL_OVERRIDES_SECTION: &str = "[Software\\\\Wine\\\\DllOverrides]";
const DLL_OVERRIDE_VALUE: &str = "native,builtin";

pub fn get_wine_prefix_path(game: &DbGame) -> Option<PathBuf> {
	let app_id = get_app_id(game)?;
	let steam_dir = match SteamDir::locate() {
		Ok(steam_dir) => steam_dir,
		Err(error) => {
			log::error!(
				"Failed to locate Steam directory while resolving compatdata for app {app_id}: {error}",
			);
			return None;
		}
	};

	let library_path = get_app_library_path(&steam_dir, app_id)?;

	Some(
		library_path
			.join("steamapps")
			.join("compatdata")
			.join(app_id.to_string())
			.join("pfx"),
	)
}

pub fn get_wine_binary_path(game: &DbGame) -> Option<PathBuf> {
	let prefix_path = get_wine_prefix_path(game)?;
	let compat_data_path = prefix_path.parent()?;
	let config_info_path = compat_data_path.join("config_info");

	let config_info_data = match fs::read_to_string(&config_info_path) {
		Ok(data) => data,
		Err(error) => {
			log::warn!(
				"Failed to read Steam Proton config_info for `{}` ({}) at {}: {}",
				game.display_title,
				game.external_id,
				config_info_path.display(),
				error,
			);
			return None;
		}
	};

	let proton_lib_path_line = match config_info_data.lines().nth(2) {
		Some(line) if !line.trim().is_empty() => line.trim(),
		_ => {
			log::warn!(
				"Steam Proton config_info for `{}` ({}) is missing a valid third line at {}",
				game.display_title,
				game.external_id,
				config_info_path.display(),
			);
			return None;
		}
	};

	let proton_lib_path = Path::new(proton_lib_path_line);
	let Some(proton_files_path) = proton_lib_path.parent() else {
		log::warn!(
			"Failed to derive Proton files path from `{}` for `{}` ({})",
			proton_lib_path_line,
			game.display_title,
			game.external_id,
		);
		return None;
	};

	Some(proton_files_path.join("bin").join("wine"))
}

pub fn run_with_wine(
	game: &DbGame,
	exe_path: &Path,
	args: &[String],
	wine_env: &HashMap<String, String>,
) -> Result {
	let wine_prefix_path = get_wine_prefix_path(game).with_context(|| {
		format!(
			"Failed to resolve Wine prefix path for Steam game `{}` ({})",
			game.display_title, game.external_id
		)
	})?;

	let wine_binary_path = get_wine_binary_path(game).with_context(|| {
		format!(
			"Failed to resolve Wine binary path for Steam game `{}` ({})",
			game.display_title, game.external_id
		)
	})?;

	let compat_data_path = wine_prefix_path.parent().with_context(|| {
		format!(
			"Failed to derive STEAM_COMPAT_DATA_PATH from Wine prefix `{}`",
			wine_prefix_path.display()
		)
	})?;

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

pub(crate) fn set_wine_dll_overrides_for_game(game: &DbGame, dll_overrides: &[String]) -> Result {
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

	if let Some(user_reg_path) = get_compat_user_reg_path(game) {
		log::info!(
			"Steam Proton user.reg resolved for `{}` ({}): {}",
			game.display_title,
			game.external_id,
			user_reg_path.display(),
		);

		upsert_dll_overrides_in_user_reg(&user_reg_path, dll_overrides)?;
	} else {
		log::warn!(
			"Failed to resolve Proton user.reg for Steam game `{}` ({})",
			game.display_title,
			game.external_id,
		);
	}

	Ok(())
}

fn get_app_id(game: &DbGame) -> Option<u32> {
	match game.external_id.parse() {
		Ok(app_id) => Some(app_id),
		Err(error) => {
			log::warn!(
				"Failed to parse Steam app ID from external_id `{}` for `{}`: {}",
				game.external_id,
				game.display_title,
				error,
			);
			None
		}
	}
}

fn get_app_library_path(steam_dir: &SteamDir, app_id: u32) -> Option<PathBuf> {
	let libraries = match steam_dir.libraries() {
		Ok(libraries) => libraries,
		Err(error) => {
			log::error!("Failed to enumerate Steam libraries: {error}");
			return None;
		}
	};

	for library_result in libraries {
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
			return Some(library.path().to_path_buf());
		}
	}

	log::warn!("Steam app {app_id} was not found in any Steam library");

	None
}

fn get_compat_user_reg_path(game: &DbGame) -> Option<PathBuf> {
	Some(get_wine_prefix_path(game)?.join("user.reg"))
}

fn upsert_dll_overrides_in_user_reg(path: &Path, dll_overrides: &[String]) -> Result {
	if !path.exists() {
		log::warn!(
			"Steam Proton user.reg does not exist yet at {}. Launch the game once to create the prefix.",
			path.display(),
		);
		return Ok(());
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
