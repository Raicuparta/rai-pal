use std::{
	collections::HashMap,
	path::PathBuf,
};

use memchr::memchr;

use crate::result::{
	Error,
	LogErrExt,
	Result,
};

const BIN_NONE: u8 = 0x00;
const BIN_STRING: u8 = 0x01;
const BIN_INT32: u8 = 0x02;
const BIN_FLOAT32: u8 = 0x03;
const BIN_POINTER: u8 = 0x04;
const BIN_WIDESTRING: u8 = 0x05;
const BIN_COLOR: u8 = 0x06;
const BIN_UINT64: u8 = 0x07;
const BIN_INT64: u8 = 0x0A;
const BIN_END_ALT: u8 = 0x0B;
const BIN_END: u8 = 0x08;

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

#[inline]
pub fn read_u32_le(data: &[u8], pos: &mut usize) -> u32 {
	let bytes = data[*pos..*pos + 4].try_into().unwrap();
	*pos += 4;
	u32::from_le_bytes(bytes)
}

#[inline]
fn read_i32_le(data: &[u8], pos: &mut usize) -> i32 {
	let bytes = data[*pos..*pos + 4].try_into().unwrap();
	*pos += 4;
	i32::from_le_bytes(bytes)
}

#[inline]
pub fn read_u64_le(data: &[u8], pos: &mut usize) -> u64 {
	let bytes = data[*pos..*pos + 8].try_into().unwrap();
	*pos += 8;
	u64::from_le_bytes(bytes)
}

#[inline]
fn read_i64_le(data: &[u8], pos: &mut usize) -> i64 {
	let bytes = data[*pos..*pos + 8].try_into().unwrap();
	*pos += 8;
	i64::from_le_bytes(bytes)
}

pub fn read_cstring(data: &[u8], pos: &mut usize) -> Result<String> {
	let remaining = &data[*pos..];
	let null_pos = memchr(0, remaining).ok_or_else(|| {
		Error::Io(std::io::Error::new(
			std::io::ErrorKind::UnexpectedEof,
			"expected null terminator in VDF string",
		))
	})?;
	let s = String::from_utf8_lossy(&remaining[..null_pos]).to_string();
	*pos += null_pos + 1;
	Ok(s)
}

fn skip_cstring(data: &[u8], pos: &mut usize) {
	let remaining = &data[*pos..];
	let null_pos = memchr(0, remaining).unwrap();
	*pos += null_pos + 1;
}

fn read_wide_string(data: &[u8], pos: &mut usize) -> Result<String> {
	let mut buf: Vec<u16> = vec![];
	loop {
		let Ok(bytes) = data[*pos..*pos + 2].try_into() else {
			return Err(Error::Io(std::io::Error::new(
				std::io::ErrorKind::UnexpectedEof,
				"unexpected end of wide string",
			)));
		};
		*pos += 2;
		let c = u16::from_le_bytes(bytes);
		if c == 0 {
			break;
		}
		buf.push(c);
	}
	Ok(String::from_utf16_lossy(&buf))
}

fn skip_wide_string(data: &[u8], pos: &mut usize) {
	loop {
		let Ok(bytes) = data[*pos..*pos + 2].try_into() else {
			return;
		};
		*pos += 2;
		let c = u16::from_le_bytes(bytes);
		if c == 0 {
			break;
		}
	}
}

fn get_key(data: &[u8], pos: &mut usize, keys: Option<&[String]>) -> Result<String> {
	match keys {
		Some(keys) => {
			let idx = usize::try_from(read_i32_le(data, pos))?;
			Ok(keys.get(idx).cloned().unwrap_or_else(|| {
				let fallback = format!("APPINFO_FALLBACK_{idx}");
				log::warn!(
					"Failed to find Steam appinfo key at index {idx}. Falling back to {fallback}"
				);
				fallback
			}))
		}
		None => read_cstring(data, pos),
	}
}

fn skip_key(data: &[u8], pos: &mut usize, keys: Option<&[String]>) {
	if keys.is_some() {
		*pos += 4;
	} else {
		skip_cstring(data, pos);
	}
}

fn skip_typed_value(data: &[u8], pos: &mut usize, t: u8) {
	match t {
		BIN_NONE => unreachable!("skip_typed_value called for BIN_NONE; use skip_vdf"),
		BIN_STRING => skip_cstring(data, pos),
		BIN_WIDESTRING => skip_wide_string(data, pos),
		BIN_INT32 | BIN_FLOAT32 | BIN_POINTER | BIN_COLOR => *pos += 4,
		BIN_UINT64 | BIN_INT64 => *pos += 8,
		_ => unreachable!("invalid VDF type byte: {t}"),
	}
}

pub fn skip_vdf(data: &[u8], pos: &mut usize, keys: Option<&[String]>) {
	loop {
		let t = data[*pos];
		*pos += 1;
		if t == BIN_END {
			return;
		}
		skip_key(data, pos, keys);
		match t {
			BIN_NONE => skip_vdf(data, pos, keys),
			_ => skip_typed_value(data, pos, t),
		}
	}
}

pub fn read_kv_mmap(
	data: &[u8],
	pos: &mut usize,
	keys_option: Option<&[String]>,
	alt_format: bool,
) -> Result<KeyValues> {
	let end_marker = if alt_format { BIN_END_ALT } else { BIN_END };
	let mut node = KeyValues::new();
	loop {
		let t = data[*pos];
		*pos += 1;
		if t == end_marker {
			return Ok(node);
		}
		let key = get_key(data, pos, keys_option)?;
		match t {
			BIN_NONE => {
				let sub = read_kv_mmap(data, pos, keys_option, alt_format)?;
				node.insert(key, ValueType::KeyValue(sub));
			}
			BIN_STRING => {
				let val = read_cstring(data, pos)?;
				node.insert(key, ValueType::String(val));
			}
			BIN_WIDESTRING => {
				let val = read_wide_string(data, pos)?;
				node.insert(key, ValueType::WideString(val));
			}
			BIN_INT32 => {
				let val = read_i32_le(data, pos);
				node.insert(key, ValueType::Int32(val));
			}
			BIN_POINTER => {
				let val = read_i32_le(data, pos);
				node.insert(key, ValueType::Pointer(val));
			}
			BIN_COLOR => {
				let val = read_i32_le(data, pos);
				node.insert(key, ValueType::Color(val));
			}
			BIN_UINT64 => {
				let val = read_u64_le(data, pos);
				node.insert(key, ValueType::UInt64(val));
			}
			BIN_INT64 => {
				let val = read_i64_le(data, pos);
				node.insert(key, ValueType::Int64(val));
			}
			BIN_FLOAT32 => {
				let val = f32::from_le_bytes(data[*pos..*pos + 4].try_into().unwrap());
				*pos += 4;
				node.insert(key, ValueType::Float32(val));
			}
			_ => return Err(Error::InvalidBinaryVdfType(t, key)),
		}
	}
}

pub fn find_app_type_in_vdf(
	data: &[u8],
	pos: &mut usize,
	keys: Option<&[String]>,
) -> Option<String> {
	let saved = *pos;

	let t = *data.get(*pos)?;
	*pos += 1;
	if t != BIN_NONE {
		*pos = saved;
		return None;
	}

	let top_key = get_key_str(data, pos, keys)?;
	if top_key != "appinfo" {
		*pos = saved;
		return None;
	}

	loop {
		let inner_t = *data.get(*pos)?;
		*pos += 1;
		if inner_t == BIN_END {
			*pos = saved;
			return None;
		}

		let section_key = get_key_str(data, pos, keys)?;
		if section_key == "common" {
			if inner_t != BIN_NONE {
				*pos = saved;
				return None;
			}
			loop {
				let field_t = *data.get(*pos)?;
				*pos += 1;
				if field_t == BIN_END {
					*pos = saved;
					return None;
				}

				let field_key = get_key_str(data, pos, keys)?;
				if field_key == "type" && field_t == BIN_STRING {
					let app_type =
						read_cstring(data, pos).ok_or_log("Failed to read app_type in VDF")?;
					return Some(app_type);
				}
				match field_t {
					BIN_NONE => skip_vdf(data, pos, keys),
					_ => skip_typed_value(data, pos, field_t),
				}
			}
		}

		match inner_t {
			BIN_NONE => skip_vdf(data, pos, keys),
			_ => skip_typed_value(data, pos, inner_t),
		}
	}
}

fn get_key_str(data: &[u8], pos: &mut usize, keys: Option<&[String]>) -> Option<String> {
	match keys {
		Some(keys) => {
			let idx = usize::try_from(read_i32_le(data, pos))
				.ok_or_log("Failed to parse VDF key index")?;
			keys.get(idx).cloned()
		}
		None => read_cstring(data, pos).ok_or_log("Failed to read VDF key string"),
	}
}
