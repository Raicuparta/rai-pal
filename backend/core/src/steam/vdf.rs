// This code is based on https://github.com/drguildo/vdfr
// It has been adapted to fit the needs of this project.

use std::{
	collections::HashMap,
	fs,
	io::BufReader,
	path::PathBuf,
};

use byteorder::{LittleEndian, ReadBytesExt};

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

pub type KeyValues = HashMap<String, ValueType>;

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

pub fn value_to_string(value: Option<&ValueType>) -> Option<String> {
	match value {
		Some(ValueType::String(string_value)) => Some(String::from(string_value)),
		_ => None,
	}
}

pub const fn value_to_i32(value: Option<&ValueType>) -> Option<i32> {
	match value {
		Some(ValueType::Int32(number_value)) => Some(*number_value),
		_ => None,
	}
}

pub fn value_to_path(value: Option<&ValueType>) -> Option<PathBuf> {
	match value {
		Some(ValueType::String(string_value)) => {
			Some(PathBuf::from(string_value.replace('\\', "/")))
		}
		_ => None,
	}
}

pub const fn value_to_kv(value: Option<&ValueType>) -> Option<&KeyValues> {
	match value {
		Some(ValueType::KeyValue(kv_value)) => Some(kv_value),
		_ => None,
	}
}

pub fn read_kv(
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

pub fn read_string(reader: &mut BufReader<fs::File>, wide: bool) -> Result<String> {
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
