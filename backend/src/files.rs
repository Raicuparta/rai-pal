use std::{
	fs,
	io,
	path::Path,
};

use zip::ZipArchive;

use crate::Result;

pub fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> Result {
	fs::create_dir_all(&dst)?;
	for entry in fs::read_dir(src)? {
		let entry = entry?;
		let file_type = entry.file_type()?;
		if file_type.is_dir() {
			copy_dir_all(entry.path(), dst.as_ref().join(entry.file_name()))?;
		} else {
			fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
		}
	}
	Ok(())
}

pub fn unzip(zip_path: &Path, target_path: &Path) -> Result {
	let zip_file = fs::File::open(zip_path)?;
	let mut archive = ZipArchive::new(zip_file)?;

	for file_index in 0..archive.len() {
		let mut file = archive.by_index(file_index)?;

		if let Some(enclosed_name) = file.enclosed_name() {
			let out_path = target_path.join(enclosed_name);

			if file.name().ends_with('/') {
				fs::create_dir_all(&out_path)?;
			} else {
				if let Some(p) = out_path.parent() {
					if !p.exists() {
						fs::create_dir_all(p)?;
					}
				}
				let mut out_file = fs::File::create(&out_path)?;
				io::copy(&mut file, &mut out_file)?;
			}
		}
	}
	Ok(())
}
