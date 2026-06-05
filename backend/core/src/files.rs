use std::{
	fs::{
		self,
		File,
	},
	io,
	path::Path,
};

use zip::ZipArchive;

use crate::result::Result;

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

pub fn extract(archive_path: &Path, target_path: &Path) -> io::Result<()> {
	let mut archive = ZipArchive::new(File::open(archive_path)?)?;

	for i in 0..archive.len() {
		let mut file = archive.by_index(i).map_err(io::Error::other)?;

		// Some zips created on windows have cursed backslashes in their paths.
		// We need to replace them with forward slashes to avoid issues when extracting on Linux.
		// Hopefully there aren't any legitimate files with backslashes in their names.
		let sanitized_name = file.name().replace('\\', "/");

		let outpath = target_path.join(&sanitized_name);

		if !outpath.starts_with(target_path) {
			return Err(io::Error::new(
				io::ErrorKind::InvalidData,
				format!("Zip file contains path escaping target: {}", file.name()),
			));
		}

		if file.is_dir() || sanitized_name.ends_with('/') {
			fs::create_dir_all(&outpath)?;
		} else {
			if let Ok(metadata) = fs::metadata(&outpath)
				&& metadata.len() == file.size()
			{
				// Skip extracting if already extracted.
				continue;
			}

			if let Some(p) = outpath.parent()
				&& !p.exists()
			{
				fs::create_dir_all(p)?;
			}
			let mut outfile = fs::File::create(&outpath)?;
			io::copy(&mut file, &mut outfile)?;
		}
	}
	Ok(())
}
