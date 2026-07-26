use std::{
	fs,
	path::{
		Path,
		PathBuf,
	},
};

use rai_pal_proc_macros::serializable_struct;

use super::vdf::{
	KeyValues,
	ValueType,
	find_keys,
	value_to_i32,
	value_to_kv,
	value_to_path,
	value_to_string,
};
use crate::result::{
	Error,
	Result,
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
	pub tags: Option<Vec<i32>>,
}

pub struct SteamAppInfoReader {
	mmap: memmap2::Mmap,
	pos: usize,
	keys: Option<Vec<String>>,
}

const OLD_APPINFO_MAX_VERSION: u32 = 0x07_56_44_28;

impl SteamAppInfoReader {
	pub fn new(appinfo_path: &Path) -> Result<Self> {
		if !appinfo_path.exists() {
			return Err(Error::SteamAppInfoNotFound(appinfo_path.to_owned()));
		}

		let file = fs::File::open(appinfo_path)?;
		let mmap = unsafe { memmap2::Mmap::map(&file)? };
		let mut pos = 0;

		let version = super::vdf::read_u32_le(&mmap, &mut pos);
		let _universe = super::vdf::read_u32_le(&mmap, &mut pos);

		let is_new_version = version > OLD_APPINFO_MAX_VERSION;

		let keys = if is_new_version {
			let key_list_address = super::vdf::read_u64_le(&mmap, &mut pos);
			let position_before_jump = pos;

			pos = usize::try_from(key_list_address)?;

			let key_count = super::vdf::read_u32_le(&mmap, &mut pos);
			let mut keys_vec: Vec<String> = Vec::with_capacity(usize::try_from(key_count)?);
			for _ in 0..key_count {
				if let Ok(key) = super::vdf::read_cstring(&mmap, &mut pos) {
					keys_vec.push(key);
				}
			}

			pos = position_before_jump;

			Some(keys_vec)
		} else {
			None
		};

		Ok(Self { mmap, pos, keys })
	}

	pub fn try_next(&mut self) -> Result<Option<SteamAppInfo>> {
		loop {
			if self.pos + 8 > self.mmap.len() {
				return Ok(None);
			}

			let app_id = super::vdf::read_u32_le(&self.mmap, &mut self.pos);
			if app_id == 0 {
				return Ok(None);
			}

			self.pos += 4; // size: u32
			self.pos += 4; // state: u32
			self.pos += 4; // last_update: u32
			self.pos += 8; // access_token: u64
			self.pos += 20; // checksum_txt: [u8; 20]
			self.pos += 4; // change_number: u32
			self.pos += 20; // checksum_bin: [u8; 20]

			let keys_ref = self.keys.as_deref();
			let vdf_start = self.pos;

			// Quick scan for app_type to skip non-game entries without full VDF parse.
			let mut scan_pos = vdf_start;
			let early_type = super::vdf::find_app_type_in_vdf(&self.mmap, &mut scan_pos, keys_ref);

			if let Some(ref app_type) = early_type
				&& app_type != "Game"
				&& app_type != "Demo"
			{
				super::vdf::skip_vdf(&self.mmap, &mut self.pos, keys_ref);
				continue;
			}

			// Full parse for kept entries (or when app_type wasn't found).
			self.pos = vdf_start;
			let key_values = super::vdf::read_kv_mmap(&self.mmap, &mut self.pos, keys_ref, false)?;

			let app = App { key_values };

			let app_launch =
				value_to_kv(app.get(&["appinfo", "config", "launch"])).and_then(|app_launch_kv| {
					let launch_map: Vec<SteamLaunchOption> = app_launch_kv
						.iter()
						.filter_map(|(key, launch)| {
							value_to_kv(Some(launch)).map(|launch_kv| SteamLaunchOption {
								launch_id: key.clone(),
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

			let tags = value_to_kv(app.get(&["appinfo", "common", "store_tags"])).map(|tag_map| {
				tag_map
					.values()
					.filter_map(|value| value_to_i32(Some(value)))
					.collect::<Vec<_>>()
			});

			let original_release_date =
				value_to_i32(app.get(&["appinfo", "common", "original_release_date"]));

			let app_type_option = value_to_string(app.get(&["appinfo", "common", "type"]));
			let is_free = app.get(&["appinfo", "extended", "isfreeapp"]).is_some()
				|| app_type_option
					.as_ref()
					.is_some_and(|app_type| app_type == "Demo");

			if app_type_option
				.as_ref()
				.is_some_and(|app_type| app_type != "Game" && app_type != "Demo")
			{
				// We don't care about things like dedicated server, sdk, videos, dlcs, etc.
				continue;
			}

			if let Some(launch_options) = app_launch
				&& let Some(name) = value_to_string(
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
					tags,
				}));
			}
		}
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
