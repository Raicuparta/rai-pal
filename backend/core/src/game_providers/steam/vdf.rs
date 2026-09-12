use std::{collections::HashMap, path::PathBuf};

use memchr::memchr;

use crate::result::{Error, Result};

fn unexpected_eof() -> Error {
	Error::Io(std::io::Error::new(
		std::io::ErrorKind::UnexpectedEof,
		"unexpected end of VDF data",
	))
}

/// Reads a fixed-size array without panicking on truncated data.
pub(super) fn read_array<const N: usize>(data: &[u8], pos: &mut usize) -> Result<[u8; N]> {
	let end = (*pos).checked_add(N).ok_or_else(unexpected_eof)?;
	let mut array = [0_u8; N];
	array.copy_from_slice(data.get(*pos..end).ok_or_else(unexpected_eof)?);
	*pos = end;
	Ok(array)
}

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
pub fn read_u32_le(data: &[u8], pos: &mut usize) -> Result<u32> {
	Ok(u32::from_le_bytes(read_array::<4>(data, pos)?))
}

#[inline]
fn read_i32_le(data: &[u8], pos: &mut usize) -> Result<i32> {
	Ok(i32::from_le_bytes(read_array::<4>(data, pos)?))
}

#[inline]
pub fn read_u64_le(data: &[u8], pos: &mut usize) -> Result<u64> {
	Ok(u64::from_le_bytes(read_array::<8>(data, pos)?))
}

#[inline]
fn read_i64_le(data: &[u8], pos: &mut usize) -> Result<i64> {
	Ok(i64::from_le_bytes(read_array::<8>(data, pos)?))
}

pub fn read_cstring(data: &[u8], pos: &mut usize) -> Result<String> {
	let remaining = data.get(*pos..).ok_or_else(unexpected_eof)?;
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

fn skip_cstring(data: &[u8], pos: &mut usize) -> Result {
	let remaining = data.get(*pos..).ok_or_else(unexpected_eof)?;
	let null_pos = memchr(0, remaining).ok_or_else(unexpected_eof)?;
	*pos += null_pos + 1;
	Ok(())
}

fn read_wide_string(data: &[u8], pos: &mut usize) -> Result<String> {
	let mut buf: Vec<u16> = vec![];
	loop {
		let c = u16::from_le_bytes(read_array::<2>(data, pos)?);
		if c == 0 {
			break;
		}
		buf.push(c);
	}
	Ok(String::from_utf16_lossy(&buf))
}

fn skip_wide_string(data: &[u8], pos: &mut usize) -> Result {
	loop {
		let c = u16::from_le_bytes(read_array::<2>(data, pos)?);
		if c == 0 {
			break;
		}
	}
	Ok(())
}

fn get_key(data: &[u8], pos: &mut usize, keys: Option<&[String]>) -> Result<String> {
	match keys {
		Some(keys) => {
			let idx = usize::try_from(read_i32_le(data, pos)?)?;
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

fn skip_key(data: &[u8], pos: &mut usize, keys: Option<&[String]>) -> Result {
	if keys.is_some() {
		*pos += 4;
		Ok(())
	} else {
		skip_cstring(data, pos)
	}
}

fn skip_typed_value(data: &[u8], pos: &mut usize, t: u8) -> Result {
	match t {
		BIN_NONE => unreachable!("skip_typed_value called for BIN_NONE; use skip_vdf"),
		BIN_STRING => skip_cstring(data, pos),
		BIN_WIDESTRING => skip_wide_string(data, pos),
		BIN_INT32 | BIN_FLOAT32 | BIN_POINTER | BIN_COLOR => {
			*pos += 4;
			Ok(())
		}
		BIN_UINT64 | BIN_INT64 => {
			*pos += 8;
			Ok(())
		}
		_ => Err(Error::InvalidBinaryVdfType(t, "<skipped>".to_string())),
	}
}

pub fn skip_vdf(data: &[u8], pos: &mut usize, keys: Option<&[String]>) -> Result {
	loop {
		let t = *data.get(*pos).ok_or_else(unexpected_eof)?;
		*pos += 1;
		if t == BIN_END {
			return Ok(());
		}
		skip_key(data, pos, keys)?;
		match t {
			BIN_NONE => skip_vdf(data, pos, keys)?,
			_ => skip_typed_value(data, pos, t)?,
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
		let t = *data.get(*pos).ok_or_else(unexpected_eof)?;
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
				let val = read_i32_le(data, pos)?;
				node.insert(key, ValueType::Int32(val));
			}
			BIN_POINTER => {
				let val = read_i32_le(data, pos)?;
				node.insert(key, ValueType::Pointer(val));
			}
			BIN_COLOR => {
				let val = read_i32_le(data, pos)?;
				node.insert(key, ValueType::Color(val));
			}
			BIN_UINT64 => {
				let val = read_u64_le(data, pos)?;
				node.insert(key, ValueType::UInt64(val));
			}
			BIN_INT64 => {
				let val = read_i64_le(data, pos)?;
				node.insert(key, ValueType::Int64(val));
			}
			BIN_FLOAT32 => {
				let val = f32::from_le_bytes(read_array::<4>(data, pos)?);
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
) -> Result<Option<String>> {
	let saved = *pos;

	let Some(&t) = data.get(*pos) else {
		return Ok(None);
	};
	*pos += 1;
	if t != BIN_NONE {
		*pos = saved;
		return Ok(None);
	}

	let Some(top_key) = get_key_str(data, pos, keys)? else {
		*pos = saved;
		return Ok(None);
	};
	if top_key != "appinfo" {
		*pos = saved;
		return Ok(None);
	}

	loop {
		let Some(&inner_t) = data.get(*pos) else {
			return Ok(None);
		};
		*pos += 1;
		if inner_t == BIN_END {
			*pos = saved;
			return Ok(None);
		}

		let Some(section_key) = get_key_str(data, pos, keys)? else {
			*pos = saved;
			return Ok(None);
		};
		if section_key == "common" {
			if inner_t != BIN_NONE {
				*pos = saved;
				return Ok(None);
			}
			loop {
				let Some(&field_t) = data.get(*pos) else {
					return Ok(None);
				};
				*pos += 1;
				if field_t == BIN_END {
					*pos = saved;
					return Ok(None);
				}

				let Some(field_key) = get_key_str(data, pos, keys)? else {
					*pos = saved;
					return Ok(None);
				};
				if field_key == "type" && field_t == BIN_STRING {
					return Ok(Some(read_cstring(data, pos)?));
				}
				match field_t {
					BIN_NONE => skip_vdf(data, pos, keys)?,
					_ => skip_typed_value(data, pos, field_t)?,
				}
			}
		}

		match inner_t {
			BIN_NONE => skip_vdf(data, pos, keys)?,
			_ => skip_typed_value(data, pos, inner_t)?,
		}
	}
}

fn get_key_str(data: &[u8], pos: &mut usize, keys: Option<&[String]>) -> Result<Option<String>> {
	match keys {
		Some(keys) => {
			let idx = usize::try_from(read_i32_le(data, pos)?)?;
			Ok(keys.get(idx).cloned())
		}
		None => Ok(Some(read_cstring(data, pos)?)),
	}
}
