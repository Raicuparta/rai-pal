// This code is based on https://github.com/drguildo/vdfr
// It has been adapted to fit the needs of this project.

use std::{
	collections::{HashMap, HashSet},
	fs,
	io::{BufReader, Read},
	path::Path,
};

use byteorder::{LittleEndian, ReadBytesExt};

use crate::result::Result;

use super::vdf::{find_keys, read_kv, KeyValues, ValueType};

#[derive(Debug)]
pub struct Package {
	pub checksum: [u8; 20],
	pub change_number: u32,
	pub pics: u64,
	pub key_values: KeyValues,
}

#[derive(Debug)]
pub struct PackageInfo {
	pub magic: u32,
	pub universe: u32,
	pub packages: HashMap<u32, Package>,
}

impl PackageInfo {
	pub fn read(path: &Path) -> Result<Self> {
		let mut reader = BufReader::new(fs::File::open(path)?);

		let magic = reader.read_u32::<LittleEndian>()?;
		let universe = reader.read_u32::<LittleEndian>()?;

		let mut package_info = Self {
			magic,
			universe,
			packages: HashMap::new(),
		};

		loop {
			let package_id = reader.read_u32::<LittleEndian>()?;

			if package_id == 0xffff_ffff {
				break;
			}

			let mut checksum: [u8; 20] = [0; 20];
			reader.read_exact(&mut checksum)?;

			let change_number = reader.read_u32::<LittleEndian>()?;

			// XXX: No idea what this is. Seems to get ignored in vdf.py.
			let pics = reader.read_u64::<LittleEndian>()?;

			let key_values = read_kv(&mut reader, false, None)?;

			let package = Package {
				checksum,
				change_number,
				pics,
				key_values,
			};

			package_info.packages.insert(package_id, package);
		}

		Ok(package_info)
	}

	pub fn get_app_ids(&self) -> HashSet<String> {
		self.packages
			.values()
			.flat_map(Package::get_app_ids)
			.collect()
	}
}

impl Package {
	pub fn get(&self, keys: &[&str]) -> Option<&ValueType> {
		find_keys(&self.key_values, keys)
	}

	pub fn get_app_ids(&self) -> HashSet<String> {
		// As far as I can tell, there's always just a single item here.
		// But just to be safe, I'm mapping over the map, just in case there are more.
		self.key_values
			.values()
			.filter_map(|value| match value {
				ValueType::KeyValue(root_value) => root_value.get("appids"),
				_ => None,
			})
			.filter_map(|app_ids| match app_ids {
				ValueType::KeyValue(app_ids) => Some(app_ids),
				_ => None,
			})
			.flat_map(|app_ids| app_ids.values())
			.filter_map(|app_id_value| match app_id_value {
				ValueType::Int32(app_id) => Some(app_id.to_string()),
				_ => None,
			})
			.collect()
	}
}
