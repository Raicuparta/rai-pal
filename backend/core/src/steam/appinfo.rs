// This code is based on https://github.com/drguildo/vdfr
// It has been adapted to fit the needs of this project.

use std::{
	collections::HashMap,
	fs,
	io::{BufReader, Read, Seek, SeekFrom},
	path::{Path, PathBuf},
};

use byteorder::{LittleEndian, ReadBytesExt};
use rai_pal_proc_macros::serializable_struct;
use steamlocate::SteamDir;

use crate::result::{Error, Result};

const BIN_NONE: u8 = b'\x00';
const BIN_STRING: u8 = b'\x01';
const BIN_INT32: u8 = b'\x02';
const BIN_FLOAT32: u8 = b'\x03';
const BIN_POINTER: u8 = b'\x04';
const BIN_WIDESTRING: u8 = b'\x05';
const BIN_COLOR: u8 = b'\x06';
const BIN_UINT64: u8 = b'\x07';
const BIN_END: u8 = b'\x08';
const BIN_INT64: u8 = b'\x0A';
const BIN_END_ALT: u8 = b'\x0B';

#[allow(dead_code)] // Some unused fields inside the types, keeping them for future reference.
#[derive(Debug)]
pub enum ValueType {
	String(String),
	WideString(String),
	Int32(i32),
	Pointer(i32),
	Color(i32),
	UInt64(u64),
	Int64(i64),
	Float32(f32),
	KeyValue(KeyValues),
}

type KeyValues = HashMap<String, ValueType>;

// Recursively search for the specified sequence of keys in the key-value data.
// The order of the keys dictates the hierarchy, with all except the last having
// to be a Value::KeyValueType.
pub fn find_keys<'a>(key_value: &'a KeyValues, keys: &[&str]) -> Option<&'a ValueType> {
	if keys.is_empty() {
		return None;
	}

	let value = key_value.get(*keys.first()?);
	if keys.len() == 1 {
		value
	} else if let Some(ValueType::KeyValue(child_key_value)) = value {
		find_keys(child_key_value, &keys[1..])
	} else {
		None
	}
}

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
			let is_free = value_to_string(app.get(&["appinfo", "extended", "isfreeapp"])).is_some()
				|| app_type_option
					.as_ref()
					.is_some_and(|app_type| app_type == "Demo");

			if app_type_option
				.clone()
				.is_some_and(|app_type| app_type == "Tool")
			{
				// We don't care about tools like dedicated server, sdk, etc.
				continue;
			}

			if let Some(launch_options) = app_launch {
				if let Some(name) = value_to_string(app.get(&["appinfo", "common", "name"])) {
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

fn value_to_string(value: Option<&ValueType>) -> Option<String> {
	match value {
		Some(ValueType::String(string_value)) => Some(String::from(string_value)),
		_ => None,
	}
}

const fn value_to_i32(value: Option<&ValueType>) -> Option<i32> {
	match value {
		Some(ValueType::Int32(number_value)) => Some(*number_value),
		_ => None,
	}
}

fn value_to_path(value: Option<&ValueType>) -> Option<PathBuf> {
	match value {
		Some(ValueType::String(string_value)) => {
			Some(PathBuf::from(string_value.replace('\\', "/")))
		}
		_ => None,
	}
}

const fn value_to_kv(value: Option<&ValueType>) -> Option<&KeyValues> {
	match value {
		Some(ValueType::KeyValue(kv_value)) => Some(kv_value),
		_ => None,
	}
}

impl App {
	pub fn get(&self, keys: &[&str]) -> Option<&ValueType> {
		find_keys(&self.key_values, keys)
	}
}

fn read_kv(
	reader: &mut BufReader<fs::File>,
	alt_format: bool,
	keys_option: Option<&Vec<String>>,
) -> Result<KeyValues> {
	let current_bin_end = if alt_format { BIN_END_ALT } else { BIN_END };

	let mut node = KeyValues::new();

	loop {
		let t = reader.read_u8()?;
		if t == current_bin_end {
			return Ok(node);
		}

		let key = if let Some(keys) = keys_option {
			let key_index = usize::try_from(reader.read_i32::<LittleEndian>()?)?;
			keys.get(key_index).cloned().unwrap_or_else(|| {
				let fallback_key = format!("APPINFO_FALLBACK_{key_index}");
				log::warn!(
					"Failed to find a Steam appinfo key at index {}. Falling back to {}",
					key_index,
					fallback_key
				);
				fallback_key
			})
		} else {
			read_string(reader, false)?
		};

		if t == BIN_NONE {
			let subnode = read_kv(reader, alt_format, keys_option)?;
			node.insert(key, ValueType::KeyValue(subnode));
		} else if t == BIN_STRING {
			let s = read_string(reader, false)?;
			node.insert(key, ValueType::String(s));
		} else if t == BIN_WIDESTRING {
			let s = read_string(reader, true)?;
			node.insert(key, ValueType::WideString(s));
		} else if [BIN_INT32, BIN_POINTER, BIN_COLOR].contains(&t) {
			let val = reader.read_i32::<LittleEndian>()?;
			if t == BIN_INT32 {
				node.insert(key, ValueType::Int32(val));
			} else if t == BIN_POINTER {
				node.insert(key, ValueType::Pointer(val));
			} else if t == BIN_COLOR {
				node.insert(key, ValueType::Color(val));
			}
		} else if t == BIN_UINT64 {
			let val = reader.read_u64::<LittleEndian>()?;
			node.insert(key, ValueType::UInt64(val));
		} else if t == BIN_INT64 {
			let val = reader.read_i64::<LittleEndian>()?;
			node.insert(key, ValueType::Int64(val));
		} else if t == BIN_FLOAT32 {
			let val = reader.read_f32::<LittleEndian>()?;
			node.insert(key, ValueType::Float32(val));
		} else {
			return Err(Error::InvalidBinaryVdfType(t, key));
		}
	}
}

fn read_string(reader: &mut BufReader<fs::File>, wide: bool) -> Result<String> {
	if wide {
		let mut buf: Vec<u16> = vec![];
		loop {
			// Maybe this should be big-endian?
			let c = reader.read_u16::<LittleEndian>()?;
			if c == 0 {
				break;
			}
			buf.push(c);
		}
		Ok(std::string::String::from_utf16_lossy(&buf))
	} else {
		let mut buf: Vec<u8> = vec![];
		loop {
			let c = reader.read_u8()?;
			if c == 0 {
				break;
			}
			buf.push(c);
		}

		Ok(std::string::String::from_utf8_lossy(&buf).to_string())
	}
}

pub fn get_path(steam_path: &Path) -> PathBuf {
	steam_path.join("appcache/appinfo.vdf")
}

pub fn delete() -> Result {
	let steam_dir = SteamDir::locate()?;
	Ok(fs::remove_file(get_path(steam_dir.path()))?)
}
