use std::{
	fs,
	path::PathBuf,
	sync::{
		LazyLock,
		atomic::{AtomicU64, Ordering},
	},
	time::UNIX_EPOCH,
};

use rai_pal_core::app_paths;
use sha2::{Digest, Sha256};

pub(crate) const THUMBNAIL_WIDTH: u32 = 200;
pub(crate) const THUMBNAIL_HEIGHT: u32 = 120;

const CACHE_VERSION: u32 = 1;
const MAX_CACHE_BYTES: u64 = 200 * 1024 * 1024;
const PRUNE_TARGET_BYTES: u64 = 180 * 1024 * 1024;

static CACHE_DIR: LazyLock<Option<PathBuf>> =
	LazyLock::new(|| app_paths::thumbnail_cache_path().ok());
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
	fs::read(&path).ok()
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

	if fs::write(&temp_path, bytes).is_err() {
		let _ = fs::remove_file(&temp_path);
		return;
	}

	if fs::rename(&temp_path, &path).is_err() {
		let _ = fs::remove_file(&temp_path);
	}
}

pub(crate) fn remove(url: &str, width: u32, height: u32) {
	if let Some(path) = cache_path(url, width, height) {
		let _ = fs::remove_file(path);
	}
}

pub(crate) fn prune() {
	let Some(dir) = CACHE_DIR.as_ref() else {
		return;
	};
	let Ok(entries) = fs::read_dir(dir) else {
		return;
	};

	let mut files = Vec::new();
	let mut total = 0u64;

	for entry in entries.flatten() {
		let Ok(metadata) = entry.metadata() else {
			continue;
		};

		if !metadata.is_file() {
			continue;
		}

		let size = metadata.len();
		let modified = metadata.modified().unwrap_or(UNIX_EPOCH);

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

		if fs::remove_file(path).is_ok() {
			total = total.saturating_sub(size);
		}
	}
}
