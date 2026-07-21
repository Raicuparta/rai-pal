#![cfg(target_os = "linux")]

use std::{
	fs,
	path::Path,
};

use log;

use crate::{
	app_paths,
	result::Result,
};

const DLL_OVERRIDES_SECTION: &str = "[Software\\\\Wine\\\\DllOverrides]";
const DLL_OVERRIDE_VALUE: &str = "native,builtin";

// TODO: make this be a mod action?
pub fn set_up_global_wine_overrides() -> Result {
	let path = app_paths::base_dirs()?.config_dir().join("environment.d");

	fs::create_dir_all(&path)?;

	fs::write(
		path.join("90-rai-pal-wine-overrides.conf"),
		"WINEDLLOVERRIDES=\"winhttp.dll=n,b\"",
	)?;

	Ok(())
}

/// Updates the user.reg file inside a Wine prefix to add DLL overrides.
/// Takes the path to the prefix root (e.g. `~/.itch/wine` or a Steam compatdata pfx).
pub fn set_wine_dll_overrides_in_reg(prefix_path: &Path, dll_overrides: &[String]) -> Result {
	let path = prefix_path.join("user.reg");

	let user_reg_data = if path.exists() {
		fs::read_to_string(&path)?
	} else {
		fs::create_dir_all(prefix_path)?;
		let initial_content = "REGEDIT4\n\n".to_string();
		fs::write(&path, &initial_content)?;
		log::info!(
			"Created new Wine user.reg at {}",
			path.display()
		);
		initial_content
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
