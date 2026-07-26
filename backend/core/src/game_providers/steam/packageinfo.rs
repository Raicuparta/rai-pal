use std::{
	collections::{
		HashMap,
		HashSet,
	},
	fs,
	path::Path,
};

use super::vdf::{
	KeyValues,
	ValueType,
	find_keys,
};
use crate::result::Result;

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
		let file = fs::File::open(path)?;
		let mmap = unsafe { memmap2::Mmap::map(&file)? };
		let mut pos = 0;

		let magic = super::vdf::read_u32_le(&mmap, &mut pos);
		let universe = super::vdf::read_u32_le(&mmap, &mut pos);

		let mut packages = HashMap::new();

		loop {
			let package_id = super::vdf::read_u32_le(&mmap, &mut pos);
			if package_id == 0xffff_ffff {
				break;
			}

			let mut checksum: [u8; 20] = [0; 20];
			checksum.copy_from_slice(&mmap[pos..pos + 20]);
			pos += 20;

			let change_number = super::vdf::read_u32_le(&mmap, &mut pos);
			// XXX: No idea what this is. Seems to get ignored in vdf.py.
			let pics = super::vdf::read_u64_le(&mmap, &mut pos);

			let key_values = super::vdf::read_kv_mmap(&mmap, &mut pos, None, false)?;

			let package = Package {
				checksum,
				change_number,
				pics,
				key_values,
			};

			packages.insert(package_id, package);
		}

		Ok(Self {
			magic,
			universe,
			packages,
		})
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
