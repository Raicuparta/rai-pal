const ELF_MAGIC: [u8; 4] = [0x7f, b'E', b'L', b'F'];

const ELFCLASS32: u8 = 1;
const ELFCLASS64: u8 = 2;
const ELFDATA2LSB: u8 = 1;

pub fn is_elf(bytes: &[u8]) -> bool {
	bytes.starts_with(&ELF_MAGIC)
}

fn u16_at(bytes: &[u8], offset: usize) -> Option<u16> {
	let end = offset.checked_add(2)?;
	Some(u16::from_le_bytes(bytes.get(offset..end)?.try_into().ok()?))
}

fn u32_at(bytes: &[u8], offset: usize) -> Option<u32> {
	let end = offset.checked_add(4)?;
	Some(u32::from_le_bytes(bytes.get(offset..end)?.try_into().ok()?))
}

fn u64_at(bytes: &[u8], offset: usize) -> Option<u64> {
	let end = offset.checked_add(8)?;
	Some(u64::from_le_bytes(bytes.get(offset..end)?.try_into().ok()?))
}

fn read_cstr(bytes: &[u8], offset: usize) -> Option<&str> {
	let rest = bytes.get(offset..)?;
	let len = rest.iter().position(|&byte| byte == 0)?;
	std::str::from_utf8(&rest[..len]).ok()
}

fn section_bytes(bytes: &[u8], offset: u64, size: u64) -> Option<&[u8]> {
	let start = usize::try_from(offset).ok()?;
	let len = usize::try_from(size).ok()?;
	bytes.get(start..start.checked_add(len)?)
}

/// Reads the `(name offset, file offset, size)` triple from the section header
/// at `index` in the section header table.
fn section_header(
	bytes: &[u8],
	class: u8,
	index: usize,
	table: usize,
	entry_size: usize,
) -> Option<(u32, u64, u64)> {
	let entry = table.checked_add(index.checked_mul(entry_size)?)?;
	let name = u32_at(bytes, entry)?;
	let (offset, size) = match class {
		ELFCLASS64 => (
			u64_at(bytes, entry.checked_add(24)?)?,
			u64_at(bytes, entry.checked_add(32)?)?,
		),
		ELFCLASS32 => (
			u64::from(u32_at(bytes, entry.checked_add(16)?)?),
			u64::from(u32_at(bytes, entry.checked_add(20)?)?),
		),
		_ => return None,
	};
	Some((name, offset, size))
}

/// Returns the contents of the first section named `name`, or `None` if the
/// file isn't a little-endian ELF with such a section.
pub fn find_section<'a>(bytes: &'a [u8], name: &str) -> Option<&'a [u8]> {
	if !is_elf(bytes) || *bytes.get(5)? != ELFDATA2LSB {
		return None;
	}

	let class = *bytes.get(4)?;
	let (table, entry_size, section_count, names_index) = match class {
		ELFCLASS64 => (
			usize::try_from(u64_at(bytes, 0x28)?).ok()?,
			usize::from(u16_at(bytes, 0x3a)?),
			usize::from(u16_at(bytes, 0x3c)?),
			usize::from(u16_at(bytes, 0x3e)?),
		),
		ELFCLASS32 => (
			usize::try_from(u32_at(bytes, 0x20)?).ok()?,
			usize::from(u16_at(bytes, 0x2e)?),
			usize::from(u16_at(bytes, 0x30)?),
			usize::from(u16_at(bytes, 0x32)?),
		),
		_ => return None,
	};

	if entry_size == 0 || names_index >= section_count {
		return None;
	}

	let (_, names_offset, names_size) =
		section_header(bytes, class, names_index, table, entry_size)?;
	let names = section_bytes(bytes, names_offset, names_size)?;

	for index in 0..section_count {
		let (name_offset, offset, size) = section_header(bytes, class, index, table, entry_size)?;
		let Some(section_name) = read_cstr(names, usize::try_from(name_offset).ok()?) else {
			continue;
		};
		if section_name == name {
			return section_bytes(bytes, offset, size);
		}
	}

	None
}
