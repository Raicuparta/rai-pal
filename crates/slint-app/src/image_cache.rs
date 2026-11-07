use std::{
	fs,
	path::PathBuf,
	sync::{
		LazyLock,
		atomic::{AtomicU64, Ordering},
	},
	time::UNIX_EPOCH,
};

use rai_pal_core::{app_paths, result::LogErrExt};
use sha2::{Digest, Sha256};

pub(crate) const THUMBNAIL_WIDTH: u32 = 200;
pub(crate) const THUMBNAIL_HEIGHT: u32 = 120;

const CACHE_VERSION: u32 = 1;
const MAX_CACHE_BYTES: u64 = 200 * 1024 * 1024;
const PRUNE_TARGET_BYTES: u64 = 180 * 1024 * 1024;

static CACHE_DIR: LazyLock<Option<PathBuf>> = LazyLock::new(|| {
	app_paths::thumbnail_cache_path().ok_or_log("Failed to get thumbnail cache path")
});
static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

fn cache_path(url: &str, width: u32, height: u32) -> Option<PathBuf> {
	let mut hasher = Sha256::new();
	hasher.update(CACHE_VERSION.to_le_bytes());
	hasher.update(width.to_le_bytes());
	hasher.update(height.to_le_bytes());
	hasher.update(url.as_bytes());

	Some(
		CACHE_DIR
			.as_ref()?
			.join(format!("{:x}.img", hasher.finalize())),
	)
}

pub(crate) fn contains(url: &str, width: u32, height: u32) -> bool {
	cache_path(url, width, height).is_some_and(|path| path.is_file())
}

pub(crate) fn load(url: &str, width: u32, height: u32) -> Option<Vec<u8>> {
	let path = cache_path(url, width, height)?;
	fs::read(&path).ok_or_log(&format!(
		"Failed to read thumbnail cache file {}",
		path.display()
	))
}

pub(crate) fn store(url: &str, width: u32, height: u32, bytes: &[u8]) {
	let Some(path) = cache_path(url, width, height) else {
		return;
	};

	let temp_path = path.with_extension(format!(
		"tmp-{}-{}",
		std::process::id(),
		TEMP_COUNTER.fetch_add(1, Ordering::Relaxed)
	));

	if let Err(err) = fs::write(&temp_path, bytes) {
		log::warn!(
			"Failed to write thumbnail cache file {}: {err}",
			temp_path.display()
		);
		if let Err(remove_err) = fs::remove_file(&temp_path) {
			log::warn!(
				"Failed to remove temporary thumbnail cache file {}: {remove_err}",
				temp_path.display()
			);
		}
		return;
	}

	if let Err(err) = fs::rename(&temp_path, &path) {
		log::warn!(
			"Failed to move thumbnail cache file {} to {}: {err}",
			temp_path.display(),
			path.display()
		);
		if let Err(remove_err) = fs::remove_file(&temp_path) {
			log::warn!(
				"Failed to remove temporary thumbnail cache file {}: {remove_err}",
				temp_path.display()
			);
		}
	}
}

pub(crate) fn remove(url: &str, width: u32, height: u32) {
	let Some(path) = cache_path(url, width, height) else {
		return;
	};

	if let Err(err) = fs::remove_file(&path) {
		log::warn!(
			"Failed to remove thumbnail cache file {}: {err}",
			path.display()
		);
	}
}

pub(crate) fn prune() {
	let Some(dir) = CACHE_DIR.as_ref() else {
		return;
	};
	let entries = match fs::read_dir(dir) {
		Ok(entries) => entries,
		Err(err) => {
			log::warn!(
				"Failed to read thumbnail cache directory {}: {err}",
				dir.display()
			);
			return;
		}
	};

	let mut files = Vec::new();
	let mut total = 0u64;

	for entry_result in entries {
		let entry = match entry_result {
			Ok(entry) => entry,
			Err(err) => {
				log::warn!("Failed to read thumbnail cache directory entry: {err}");
				continue;
			}
		};
		let metadata = match entry.metadata() {
			Ok(metadata) => metadata,
			Err(err) => {
				log::warn!(
					"Failed to read thumbnail cache metadata for {}: {err}",
					entry.path().display()
				);
				continue;
			}
		};

		if !metadata.is_file() {
			continue;
		}

		let size = metadata.len();
		let modified = metadata.modified().unwrap_or_else(|err| {
			log::warn!(
				"Failed to read thumbnail cache modification time for {}: {err}",
				entry.path().display()
			);
			UNIX_EPOCH
		});

		total = total.saturating_add(size);
		files.push((modified, entry.path(), size));
	}

	if total <= MAX_CACHE_BYTES {
		return;
	}

	files.sort_by_key(|(modified, _, _)| *modified);

	for (_, path, size) in files {
		if total <= PRUNE_TARGET_BYTES {
			break;
		}

		match fs::remove_file(&path) {
			Ok(()) => total = total.saturating_sub(size),
			Err(err) => log::warn!(
				"Failed to remove pruned thumbnail cache file {}: {err}",
				path.display()
			),
		}
	}
}
