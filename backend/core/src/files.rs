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

pub async fn copy_dir_all(src: &Path, dst: &Path) -> Result {
	tokio::fs::create_dir_all(dst).await?;
	let mut dirs = vec![(src.to_path_buf(), dst.to_path_buf())];

	while let Some((current_src, current_dst)) = dirs.pop() {
		let mut entries = tokio::fs::read_dir(&current_src).await?;
		while let Some(entry) = entries.next_entry().await? {
			let file_type = entry.file_type().await?;
			let entry_dst = current_dst.join(entry.file_name());
			if file_type.is_dir() {
				tokio::fs::create_dir_all(&entry_dst).await?;
				dirs.push((entry.path(), entry_dst));
			} else {
				tokio::fs::copy(entry.path(), entry_dst).await?;
			}
		}
	}
	Ok(())
}

pub fn extract(
	archive_path: &Path,
	target_path: &Path,
	on_progress: &impl Fn(u64, u64),
) -> io::Result<()> {
	let mut archive = ZipArchive::new(File::open(archive_path)?)?;

	let total_uncompressed: u64 = {
		let mut total = 0;
		for i in 0..archive.len() {
			if let Ok(file) = archive.by_index(i) {
				total += file.size();
			}
		}
		total
	};

	let mut extracted_bytes: u64 = 0;

	for i in 0..archive.len() {
		let mut file = archive.by_index(i).map_err(io::Error::other)?;
		let file_size = file.size();

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
		} else if let Ok(metadata) = fs::metadata(&outpath)
			&& metadata.len() == file_size
		{
			// Skip extracting if already extracted.
			extracted_bytes += file_size;
		} else {
			if let Some(p) = outpath.parent()
				&& !p.exists()
			{
				fs::create_dir_all(p)?;
			}
			let mut outfile = fs::File::create(&outpath)?;
			io::copy(&mut file, &mut outfile)?;
			extracted_bytes += file_size;
		}

		on_progress(extracted_bytes, total_uncompressed);
	}
	Ok(())
}
