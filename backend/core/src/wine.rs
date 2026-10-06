#![cfg(target_os = "linux")]

use std::{
	collections::BTreeMap,
	fs,
	path::{Path, PathBuf},
};

use log;

use crate::{
	game_launch::{GameLaunch, find_in_path},
	result::Result,
};

const DLL_OVERRIDES_SECTION: &str = "[Software\\\\Wine\\\\DllOverrides]";
const DLL_OVERRIDE_VALUE: &str = "native,builtin";

pub fn get_default_wine_prefix() -> PathBuf {
	std::env::var_os("WINEPREFIX").map_or_else(
		|| {
			let home = std::env::var_os("HOME").unwrap_or_default();
			PathBuf::from(home).join(".wine")
		},
		PathBuf::from,
	)
}

pub fn make_wine_launch(
	wine: &Path,
	prefix: &Path,
	environment: &BTreeMap<String, String>,
) -> GameLaunch {
	let mut launch = fhs_wrapper(wine).map_or_else(
		|| GameLaunch::new(wine),
		|wrapper| {
			let mut launch = GameLaunch::new(wrapper);
			launch.arg(wine);
			launch
		},
	);

	launch.env("WINEPREFIX", prefix);

	if let Some(wineserver) = wine
		.parent()
		.map(|parent| parent.join("wineserver"))
		.filter(|wineserver| wineserver.exists())
	{
		launch.env("WINESERVER", wineserver);
	}

	launch.envs(environment);

	launch
}

fn fhs_wrapper(wine: &Path) -> Option<PathBuf> {
	let files_dir = wine.parent()?.parent()?;

	if !files_dir.join("lib/wine/i386-unix/wine").exists()
		|| Path::new("/lib/ld-linux.so.2").exists()
	{
		return None;
	}

	find_in_path("steam-run")
}

pub fn find_system_wine() -> PathBuf {
	let wine_name = "wine";

	if let Some(path_var) = std::env::var_os("PATH") {
		for dir in std::env::split_paths(&path_var) {
			let wine_bin = dir.join(wine_name);
			if wine_bin.exists() {
				log::info!("Found wine via PATH: `{}`", wine_bin.display());
				return wine_bin;
			}
		}
	}

	let flatpak_candidates: &[&str] = &[
		"/var/lib/flatpak/app/org.winehq.Wine/current/active/files/bin/wine",
		"/var/lib/flatpak/app/org.winehq.Wine.Stable/current/active/files/bin/wine",
		"/var/lib/flatpak/app/org.winehq.Wine.Devel/current/active/files/bin/wine",
	];

	for candidate in flatpak_candidates {
		let path = PathBuf::from(candidate);
		if path.exists() {
			log::info!("Found flatpak wine: `{}`", path.display());
			return path;
		}
	}

	if let Some(home) = std::env::var_os("HOME") {
		let user_flatpak_base = PathBuf::from(home).join(".local/share/flatpak/app");
		for flatpak_id in [
			"org.winehq.Wine",
			"org.winehq.Wine.Stable",
			"org.winehq.Wine.Devel",
		] {
			let candidate = user_flatpak_base
				.join(flatpak_id)
				.join("current/active/files/bin/wine");
			if candidate.exists() {
				log::info!("Found user flatpak wine: `{}`", candidate.display());
				return candidate;
			}
		}
	}

	log::warn!("Could not find `wine` on PATH or as flatpak. Falling back to bare name.");
	PathBuf::from(wine_name)
}

pub fn find_itch_wine() -> PathBuf {
	let wine_name = "wine";

	let flatpak_wine =
		PathBuf::from("/var/lib/flatpak/app/io.itch.itch/current/active/files/bin/wine");

	if flatpak_wine.exists() {
		log::info!("Found itch flatpak wine: `{}`", flatpak_wine.display());
		return flatpak_wine;
	}

	if let Some(path_var) = std::env::var_os("PATH") {
		for dir in std::env::split_paths(&path_var) {
			let wine_bin = dir.join(wine_name);
			if wine_bin.exists() {
				log::info!("Found wine via PATH: `{}`", wine_bin.display());
				return wine_bin;
			}
		}
	}

	log::warn!("Could not find `wine` on PATH or in itch flatpak. Falling back to bare name.");
	PathBuf::from(wine_name)
}

/// Updates the user.reg file inside a Wine prefix to add DLL overrides.
/// Takes the path to the prefix root (e.g. `~/.itch/wine` or a Steam compatdata pfx).
pub fn set_wine_dll_overrides_in_reg(prefix_path: &Path, dll_overrides: &[String]) -> Result {
	let path = prefix_path.join("user.reg");

	let user_reg_data = if path.exists() {
		fs::read_to_string(&path)?
	} else {
		return Err(crate::result::Error::WinePrefixNotInitialized(
			prefix_path.to_path_buf(),
		));
	};
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

		fs::copy(&path, backup_path)?;
		fs::write(&path, ensured_user_reg_data)?;

		log::info!("Updated Wine user.reg at {}", path.display());
	}

	Ok(())
}

pub fn normalize_dll_override_name(dll_name: &str) -> String {
	if dll_name.len() > 4 && dll_name.to_ascii_lowercase().ends_with(".dll") {
		dll_name[..dll_name.len() - 4].to_string()
	} else {
		dll_name.to_string()
	}
}

pub fn reg_add_in_section(reg_data: &str, section: &str, key: &str, value: &str) -> String {
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
