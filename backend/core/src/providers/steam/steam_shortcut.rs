use std::{
	cmp::Reverse,
	fs,
	path::{
		Path,
		PathBuf,
	},
	time::SystemTime,
};

use steamlocate::SteamDir;

use crate::result::{
	Error,
	Result,
};

const RAI_PAL_SHORTCUT_NAME: &str = "Rai Pal";

#[derive(Debug, Clone, Default)]
struct ShortcutSummary {
	app_name: Option<String>,
	exe: Option<String>,
	start_byte: usize,
	end_byte: usize,
}
pub fn add_current_executable_to_steam_shortcuts(executable_path: &Path) -> Result {
	let steam_dir = SteamDir::locate()?;
	let shortcuts_paths = get_target_shortcuts_paths(steam_dir.path())?;
	log::info!(
		"Resolved Steam shortcuts.vdf paths for adding Rai Pal shortcut: {}",
		shortcuts_paths
			.iter()
			.map(|path| path.display().to_string())
			.collect::<Vec<_>>()
			.join(", ")
	);

	// AppImage Support:
	// AppImages run from a temporary `/tmp/.mount_...` directory.
	// The `APPIMAGE` env var holds the path to the actual `.AppImage` file.
	let actual_executable =
		std::env::var_os("APPIMAGE").map_or_else(|| executable_path.to_path_buf(), PathBuf::from);

	let quoted_executable = quote_path(&actual_executable);
	let start_dir = actual_executable
		.parent()
		.map_or_else(|| "\"./\"".to_string(), quote_path);
	let app_id = calculate_shortcut_app_id(RAI_PAL_SHORTCUT_NAME, &quoted_executable);

	for shortcuts_path in shortcuts_paths {
		add_shortcut_to_path(
			&shortcuts_path,
			RAI_PAL_SHORTCUT_NAME,
			&quoted_executable,
			&start_dir,
			app_id,
		)?;
	}

	Ok(())
}

fn add_shortcut_to_path(
	shortcuts_path: &Path,
	app_name: &str,
	executable: &str,
	start_dir: &str,
	app_id: u32,
) -> Result {
	if let Some(parent) = shortcuts_path.parent() {
		fs::create_dir_all(parent)?;
	}

	let mut shortcuts_bytes = if shortcuts_path.exists() {
		fs::read(shortcuts_path)?
	} else {
		new_shortcuts_file_bytes()
	};

	let (mut max_index, entries) = parse_shortcuts_file(&shortcuts_bytes)?;

	let matching_entries: Vec<_> = entries
		.iter()
		.filter(|entry| entry.app_name.as_deref() == Some(app_name))
		.collect();

	let mut ranges_to_remove: Vec<(usize, usize)> = matching_entries
		.iter()
		.map(|entry| (entry.start_byte, entry.end_byte))
		.collect();
	ranges_to_remove.sort_by_key(|b| Reverse(b.0));
	for (start, end) in &ranges_to_remove {
		shortcuts_bytes.drain(*start..*end);
	}

	max_index += 1;

	append_shortcut_entry(
		&mut shortcuts_bytes,
		max_index,
		app_name,
		executable,
		start_dir,
		app_id,
	);

	fs::write(shortcuts_path, shortcuts_bytes)?;

	Ok(())
}

fn get_target_shortcuts_paths(steam_path: &Path) -> Result<Vec<PathBuf>> {
	let userdata_path = steam_path.join("userdata");

	let candidates = fs::read_dir(userdata_path)?
		.filter_map(std::result::Result::ok)
		.filter(|entry| entry.path().is_dir())
		.filter_map(|entry| {
			let path = entry.path();
			let user_id = path.file_name()?.to_str()?;

			if !user_id.chars().all(|c| c.is_ascii_digit()) {
				return None;
			}

			let shortcuts_path = path.join("config/shortcuts.vdf");
			let sort_time = path
				.join("config/localconfig.vdf")
				.metadata()
				.and_then(|m| m.modified())
				.unwrap_or_else(|_| {
					entry
						.metadata()
						.and_then(|m| m.modified())
						.unwrap_or(SystemTime::UNIX_EPOCH)
				});

			Some((shortcuts_path, sort_time))
		})
		.collect::<Vec<_>>();

	if candidates.is_empty() {
		return Err(Error::DataEntryNotFound(
			"No Steam userdata user folder was found".to_string(),
		));
	}

	Ok(candidates.into_iter().map(|(path, _)| path).collect())
}

fn parse_shortcuts_file(bytes: &[u8]) -> Result<(u32, Vec<ShortcutSummary>)> {
	if bytes.len() < 2 {
		return Ok((0, Vec::new()));
	}

	let mut position = 0;
	if bytes.get(position).copied() != Some(0) {
		return Err(Error::InvalidBinaryVdfType(bytes[0], "<root>".to_string()));
	}
	position += 1;

	let root_name = read_cstring(bytes, &mut position)?;
	if root_name != "shortcuts" {
		return Err(Error::DataEntryNotFound(format!(
			"Steam shortcuts root object not found (got `{root_name}`)"
		)));
	}

	let mut max_index = 0;
	let mut entries = Vec::new();

	loop {
		let entry_start = position;
		let field_type = *bytes.get(position).unwrap_or(&8);
		position += 1;

		if field_type == 8 {
			break;
		}

		let field_name = read_cstring(bytes, &mut position)?;
		if field_type == 0 {
			let (mut entry, next_position) = parse_shortcut_object(bytes, position)?;
			entry.start_byte = entry_start;
			entry.end_byte = next_position;
			position = next_position;

			if let Ok(index) = field_name.parse::<u32>() {
				max_index = max_index.max(index);
			}

			entries.push(entry);
		} else {
			return Err(Error::InvalidBinaryVdfType(field_type, field_name));
		}
	}

	Ok((max_index, entries))
}

fn parse_shortcut_object(bytes: &[u8], mut position: usize) -> Result<(ShortcutSummary, usize)> {
	let mut entry = ShortcutSummary::default();

	loop {
		let field_type = *bytes.get(position).unwrap_or(&8);
		position += 1;

		if field_type == 8 {
			break;
		}

		let field_name = read_cstring(bytes, &mut position)?;
		match field_type {
			0 => position = skip_object(bytes, position)?,
			1 => {
				let value = read_cstring(bytes, &mut position)?;
				if field_name.eq_ignore_ascii_case("appname") {
					entry.app_name = Some(value);
				} else if field_name.eq_ignore_ascii_case("exe") {
					entry.exe = Some(value);
				}
			}
			2 => {
				if position + 4 > bytes.len() {
					return Err(Error::InvalidBinaryVdfType(2, field_name));
				}
				position += 4;
			}
			other => return Err(Error::InvalidBinaryVdfType(other, field_name)),
		}
	}

	Ok((entry, position))
}

fn skip_object(bytes: &[u8], mut position: usize) -> Result<usize> {
	loop {
		let field_type = *bytes.get(position).unwrap_or(&8);
		position += 1;

		if field_type == 8 {
			return Ok(position);
		}

		let field_name = read_cstring(bytes, &mut position)?;
		match field_type {
			0 => position = skip_object(bytes, position)?,
			1 => {
				read_cstring(bytes, &mut position)?;
			}
			2 => {
				if position + 4 > bytes.len() {
					return Err(Error::InvalidBinaryVdfType(2, field_name));
				}
				position += 4;
			}
			other => return Err(Error::InvalidBinaryVdfType(other, field_name)),
		}
	}
}

fn read_cstring(bytes: &[u8], position: &mut usize) -> Result<String> {
	let start = *position;
	while *position < bytes.len() {
		if bytes[*position] == 0 {
			let value = String::from_utf8_lossy(&bytes[start..*position]).into_owned();
			*position += 1;
			return Ok(value);
		}
		*position += 1;
	}

	Err(Error::InvalidBinaryVdfType(1, "<cstring>".to_string()))
}

fn append_shortcut_entry(
	shortcuts_bytes: &mut Vec<u8>,
	index: u32,
	app_name: &str,
	executable: &str,
	start_dir: &str,
	app_id: u32,
) {
	// A valid shortcuts.vdf ends with TWO 0x08 bytes (one for the `shortcuts` dictionary,
	// and one as the EOF marker). We must pop EXACTLY TWO to append INSIDE the root dictionary.
	// If we use a while loop, we accidentally pop the closing bytes of the previous shortcut too!
	for _ in 0_i32..2_i32 {
		if shortcuts_bytes.last().copied() == Some(8) {
			shortcuts_bytes.pop();
		}
	}

	push_object_start(shortcuts_bytes, &index.to_string());
	push_u32_field(shortcuts_bytes, "appid", app_id);
	push_string_field(shortcuts_bytes, "appname", app_name);
	push_string_field(shortcuts_bytes, "Exe", executable);
	push_string_field(shortcuts_bytes, "StartDir", start_dir);
	push_string_field(shortcuts_bytes, "icon", "");
	push_i32_field(shortcuts_bytes, "IsHidden", 0);
	push_i32_field(shortcuts_bytes, "AllowDesktopConfig", 1);
	push_i32_field(shortcuts_bytes, "AllowOverlay", 0);
	push_string_field(shortcuts_bytes, "FlatpakAppID", "");
	push_object_end(shortcuts_bytes); // ends the index entry
	push_object_end(shortcuts_bytes); // ends the `shortcuts` root object
	push_object_end(shortcuts_bytes); // additionally required EOF terminator byte
}

fn push_string_field(bytes: &mut Vec<u8>, key: &str, value: &str) {
	bytes.push(1);
	push_cstring(bytes, key);
	push_cstring(bytes, value);
}

fn push_i32_field(bytes: &mut Vec<u8>, key: &str, value: i32) {
	bytes.push(2);
	push_cstring(bytes, key);
	bytes.extend_from_slice(&value.to_le_bytes());
}

fn push_u32_field(bytes: &mut Vec<u8>, key: &str, value: u32) {
	bytes.push(2);
	push_cstring(bytes, key);
	bytes.extend_from_slice(&value.to_le_bytes());
}

fn push_object_start(bytes: &mut Vec<u8>, key: &str) {
	bytes.push(0);
	push_cstring(bytes, key);
}

fn push_object_end(bytes: &mut Vec<u8>) {
	bytes.push(8);
}

fn push_cstring(bytes: &mut Vec<u8>, value: &str) {
	bytes.extend_from_slice(value.as_bytes());
	bytes.push(0);
}

fn new_shortcuts_file_bytes() -> Vec<u8> {
	let mut bytes = Vec::new();
	push_object_start(&mut bytes, "shortcuts");
	push_object_end(&mut bytes);
	push_object_end(&mut bytes); // Extra EOF marker byte
	bytes
}

fn quote_path(path: &Path) -> String {
	format!("\"{path}\"", path = path.to_string_lossy())
}

fn calculate_shortcut_app_id(app_name: &str, executable: &str) -> u32 {
	let shortcut = steamlocate::shortcut::Shortcut::new(
		0,
		app_name.to_string(),
		executable.to_string(),
		String::new(),
	);

	u32::try_from(shortcut.steam_id() >> 32).unwrap_or_default()
}
