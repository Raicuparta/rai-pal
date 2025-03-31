// This code is based on https://github.com/drguildo/vdfr
// It has been adapted to fit the needs of this project.

use std::{
	fs,
	io::{BufReader, Read, Seek, SeekFrom},
	path::{Path, PathBuf},
};

use byteorder::{LittleEndian, ReadBytesExt};
use rai_pal_proc_macros::serializable_struct;
use steamlocate::SteamDir;

use crate::result::Result;

use super::vdf::{
	KeyValues, ValueType, find_keys, read_kv, read_string, value_to_i32, value_to_kv,
	value_to_path, value_to_string,
};

#[serializable_struct]
pub struct SteamLaunchOption {
	pub launch_id: String,
	pub app_id: u32,
	pub description: Option<String>,
	pub executable: Option<PathBuf>,
	pub arguments: Option<String>,
	pub launch_type: Option<String>,
	pub os_list: Option<String>,
	pub beta_key: Option<String>,
	pub os_arch: Option<String>,
}

impl SteamLaunchOption {
	pub fn is_vr(&self) -> bool {
		matches!(self.launch_type.as_deref(), Some("vr"))
	}
}

#[derive(Debug)]
pub struct App {
	pub key_values: KeyValues,
}

#[derive(Debug, Clone)]
pub struct SteamAppInfo {
	pub app_id: u32,
	pub launch_options: Vec<SteamLaunchOption>,
	pub name: String,
	pub steam_release_date: Option<i32>,
	pub original_release_date: Option<i32>,
	pub is_free: bool,
	pub app_type: Option<String>,
}

pub struct SteamAppInfoReader {
	pub reader: BufReader<fs::File>,
	pub keys: Option<Vec<String>>,
}

const OLD_APPINFO_MAX_VERSION: u32 = 0x07_56_44_28;

impl SteamAppInfoReader {
	pub fn new(appinfo_path: &Path) -> Result<Self> {
		let mut reader = BufReader::new(fs::File::open(appinfo_path)?);

		let version = reader.read_u32::<LittleEndian>()?;
		let _universe = reader.read_u32::<LittleEndian>()?;

		let is_new_version = version > OLD_APPINFO_MAX_VERSION;

		let keys = if is_new_version {
			let key_list_address = reader.read_u64::<LittleEndian>()?;

			let position_before_jump = reader.stream_position()?;

			// This new version of appinfo has all the keyvalue keys at the end of the file,
			// starting at the address given at the start.
			reader.seek(SeekFrom::Start(key_list_address))?;

			let key_count = reader.read_u32::<LittleEndian>()?;

			let mut keys: Vec<String> = Vec::with_capacity(usize::try_from(key_count)?);
			// loop key_count times and push into keys:
			for _ in 0..key_count {
				if let Ok(key) = read_string(&mut reader, false) {
					keys.push(key);
				}
			}

			// Now we jump back to the start and do what we did before.
			reader.seek(SeekFrom::Start(position_before_jump))?;

			Some(keys)
		} else {
			None
		};

		Ok(Self { reader, keys })
	}

	pub fn try_next(&mut self) -> Result<Option<SteamAppInfo>> {
		loop {
			let app_id = self.reader.read_u32::<LittleEndian>()?;

			if app_id == 0 {
				break;
			}

			let _size = self.reader.read_u32::<LittleEndian>()?;
			let _state = self.reader.read_u32::<LittleEndian>()?;
			let _last_update = self.reader.read_u32::<LittleEndian>()?;
			let _access_token = self.reader.read_u64::<LittleEndian>()?;

			let mut checksum_txt: [u8; 20] = [0; 20];
			self.reader.read_exact(&mut checksum_txt)?;

			let _change_number = self.reader.read_u32::<LittleEndian>()?;

			let mut checksum_bin: [u8; 20] = [0; 20];
			self.reader.read_exact(&mut checksum_bin)?;

			// let some_pre_kv_thing = self.reader.read_u64::<LittleEndian>()?;

			let key_values = read_kv(&mut self.reader, false, self.keys.as_ref())?;

			let app = App { key_values };

			let app_launch =
				value_to_kv(app.get(&["appinfo", "config", "launch"])).and_then(|app_launch_kv| {
					let launch_map: Vec<SteamLaunchOption> = app_launch_kv
						.iter()
						.filter_map(|(key, launch)| {
							value_to_kv(Some(launch)).map(|launch_kv| SteamLaunchOption {
								launch_id: key.to_string(),
								app_id,
								description: value_to_string(find_keys(
									launch_kv,
									&["description"],
								)),
								launch_type: value_to_string(find_keys(launch_kv, &["type"])),
								executable: value_to_path(find_keys(launch_kv, &["executable"])),
								arguments: value_to_string(find_keys(launch_kv, &["arguments"])),
								os_list: value_to_string(find_keys(
									launch_kv,
									&["config", "oslist"],
								)),
								beta_key: value_to_string(find_keys(
									launch_kv,
									&["config", "betakey"],
								)),
								os_arch: value_to_string(find_keys(
									launch_kv,
									&["config", "osarch"],
								)),
							})
						})
						.collect();

					if launch_map.is_empty() {
						None
					} else {
						Some(launch_map)
					}
				});

			let steam_release_date =
				value_to_i32(app.get(&["appinfo", "common", "steam_release_date"]));

			let original_release_date =
				value_to_i32(app.get(&["appinfo", "common", "original_release_date"]));

			let app_type_option = value_to_string(app.get(&["appinfo", "common", "type"]));
			let is_free = app.get(&["appinfo", "extended", "isfreeapp"]).is_some()
				|| app_type_option
					.as_ref()
					.is_some_and(|app_type| app_type == "Demo");

			if app_type_option
				.clone()
				.is_some_and(|app_type| app_type != "Game")
			{
				// We don't care about things like dedicated server, sdk, videos, dlcs, etc.
				continue;
			}

			if let Some(launch_options) = app_launch {
				if let Some(name) = value_to_string(
					app.get(&["appinfo", "common", "name_localized", "english"])
						.or_else(|| app.get(&["appinfo", "common", "name"])),
				) {
					return Ok(Some(SteamAppInfo {
						app_id,
						launch_options,
						name,
						steam_release_date,
						original_release_date,
						is_free,
						app_type: app_type_option,
					}));
				}
			}
		}

		Ok(None)
	}
}

impl Iterator for SteamAppInfoReader {
	type Item = Result<SteamAppInfo>;

	fn next(&mut self) -> Option<Self::Item> {
		self.try_next().transpose()
	}
}

impl App {
	pub fn get(&self, keys: &[&str]) -> Option<&ValueType> {
		find_keys(&self.key_values, keys)
	}
}

pub fn get_path(steam_path: &Path) -> PathBuf {
	steam_path.join("appcache/appinfo.vdf")
}

pub fn delete() -> Result {
	let steam_dir = SteamDir::locate()?;
	Ok(fs::remove_file(get_path(steam_dir.path()))?)
}
