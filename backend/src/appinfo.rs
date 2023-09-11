use byteorder::{LittleEndian, ReadBytesExt};
use serde::Serialize;
use specta::Type;
use std::{collections::HashMap, fs, io::BufReader, io::Error, path::PathBuf};

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

#[derive(Debug)]
pub enum VdfrError {
    InvalidType(u8),
    ReadError(std::io::Error),
}

impl std::error::Error for VdfrError {}

impl std::fmt::Display for VdfrError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            VdfrError::InvalidType(t) => write!(f, "Invalid type {t:#x}"),
            VdfrError::ReadError(e) => e.fmt(f),
        }
    }
}

impl From<std::io::Error> for VdfrError {
    fn from(e: std::io::Error) -> Self {
        VdfrError::ReadError(e)
    }
}

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
    KeyValue(KeyValue),
}

type KeyValue = HashMap<String, ValueType>;

// Recursively search for the specified sequence of keys in the key-value data.
// The order of the keys dictates the hierarchy, with all except the last having
// to be a Value::KeyValueType.
pub fn find_keys<'a>(kv: &'a KeyValue, keys: &[&str]) -> Option<&'a ValueType> {
    if keys.is_empty() {
        return None;
    }

    let key = keys.first().unwrap();
    let value = kv.get(&key.to_string());
    if keys.len() == 1 {
        value
    } else if let Some(ValueType::KeyValue(kv)) = value {
        find_keys(kv, &keys[1..])
    } else {
        None
    }
}

#[derive(Debug, Serialize, Type, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SteamLaunchOption {
    pub launch_id: String,
    pub app_id: u32,
    pub description: Option<String>,
    pub executable: Option<PathBuf>,
    pub arguments: Option<String>,
    pub app_type: Option<String>,
    pub os_list: Option<String>,
    pub beta_key: Option<String>,
    pub os_arch: Option<String>,
}

#[derive(Debug)]
pub struct App {
    pub size: u32,
    pub state: u32,
    pub last_update: u32,
    pub access_token: u64,
    pub checksum_txt: [u8; 20],
    pub checksum_bin: [u8; 20],
    pub change_number: u32,
    pub key_values: KeyValue,
}

#[derive(Debug)]
pub struct SteamAppInfo {
    pub launch_options: Vec<SteamLaunchOption>,
    pub name: String,
}

#[derive(Debug)]
pub struct SteamAppInfoFile {
    pub version: u32,
    pub universe: u32,
    pub apps: HashMap<u32, SteamAppInfo>,
}

impl SteamAppInfoFile {
    pub fn load<R: std::io::Read>(reader: &mut R) -> Result<SteamAppInfoFile, VdfrError> {
        let version = reader.read_u32::<LittleEndian>()?;
        let universe = reader.read_u32::<LittleEndian>()?;

        let mut appinfo = SteamAppInfoFile {
            universe,
            version,
            apps: HashMap::new(),
        };

        loop {
            let app_id = reader.read_u32::<LittleEndian>()?;
            if app_id == 0 {
                break;
            }

            let size = reader.read_u32::<LittleEndian>()?;
            let state = reader.read_u32::<LittleEndian>()?;
            let last_update = reader.read_u32::<LittleEndian>()?;
            let access_token = reader.read_u64::<LittleEndian>()?;

            let mut checksum_txt: [u8; 20] = [0; 20];
            reader.read_exact(&mut checksum_txt)?;

            let change_number = reader.read_u32::<LittleEndian>()?;

            let mut checksum_bin: [u8; 20] = [0; 20];
            reader.read_exact(&mut checksum_bin)?;

            let key_values = read_kv(reader, false)?;

            let app = App {
                size,
                state,
                last_update,
                access_token,
                checksum_txt,
                checksum_bin,
                change_number,
                key_values,
            };

            let app_launch = if let Some(app_launch_kv) =
                value_to_kv(app.get(&["appinfo", "config", "launch"]))
            {
                let launch_map: Vec<SteamLaunchOption> = app_launch_kv
                    .iter()
                    .filter_map(|(key, launch)| {
                        value_to_kv(Some(launch)).map(|launch_kv| SteamLaunchOption {
                            launch_id: key.to_string(),
                            app_id,
                            description: value_to_string(find_keys(launch_kv, &["description"])),
                            app_type: value_to_string(find_keys(launch_kv, &["type"])),
                            executable: value_to_path(find_keys(launch_kv, &["executable"])),
                            arguments: value_to_string(find_keys(launch_kv, &["arguments"])),
                            os_list: value_to_string(find_keys(launch_kv, &["config", "oslist"])),
                            beta_key: value_to_string(find_keys(launch_kv, &["config", "betakey"])),
                            os_arch: value_to_string(find_keys(launch_kv, &["config", "osarch"])),
                        })
                    })
                    .collect();

                if launch_map.is_empty() {
                    None
                } else {
                    Some(launch_map)
                }
            } else {
                None
            };

            if let Some(launch_options) = app_launch {
                if let Some(name) = value_to_string(app.get(&["appinfo", "common", "name"])) {
                    appinfo.apps.insert(
                        app_id,
                        SteamAppInfo {
                            launch_options,
                            name,
                        },
                    );
                }
            }
        }

        Ok(appinfo)
    }
}

fn value_to_string(value: Option<&ValueType>) -> Option<String> {
    match value {
        Some(ValueType::String(string_value)) => Some(String::from(string_value)),
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

fn value_to_kv(value: Option<&ValueType>) -> Option<&KeyValue> {
    match value {
        Some(ValueType::KeyValue(kv_value)) => Some(kv_value.clone()),
        _ => None,
    }
}

impl App {
    pub fn get(&self, keys: &[&str]) -> Option<&ValueType> {
        find_keys(&self.key_values, keys)
    }
}

fn read_kv<R: std::io::Read>(reader: &mut R, alt_format: bool) -> Result<KeyValue, VdfrError> {
    let current_bin_end = if alt_format { BIN_END_ALT } else { BIN_END };

    let mut node = KeyValue::new();

    loop {
        let t = reader.read_u8()?;
        if t == current_bin_end {
            return Ok(node);
        }

        let key = read_string(reader, false)?;

        if t == BIN_NONE {
            let subnode = read_kv(reader, alt_format)?;
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
            return Err(VdfrError::InvalidType(t));
        }
    }
}

fn read_string<R: std::io::Read>(reader: &mut R, wide: bool) -> Result<String, Error> {
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
        Ok(std::string::String::from_utf16_lossy(&buf).to_string())
    } else {
        let mut buf: Vec<u8> = vec![];
        loop {
            let c = reader.read_u8()?;
            if c == 0 {
                break;
            }
            buf.push(c);
        }
        return Ok(std::string::String::from_utf8_lossy(&buf).to_string());
    }
}

pub fn read_appinfo(path: &str) -> SteamAppInfoFile {
    let mut appinfo_file =
        BufReader::new(fs::File::open(path).unwrap_or_else(|_| panic!("Failed to read {path}")));
    let appinfo = SteamAppInfoFile::load(&mut appinfo_file);
    appinfo.unwrap()
}
