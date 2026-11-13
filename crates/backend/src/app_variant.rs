use std::{
	io::{Read as _, Seek as _, SeekFrom},
	path::Path,
};

use fs_err::File;
use rai_pal_proc_macros::serde_enum;

const PORTABLE_MARKER: &[u8] = b"RAI_PAL_PORTABLE";

#[serde_enum]
pub enum AppVariant {
	WindowsNsis,
	WindowsPortable,
	LinuxAppImage,
	MacosBundle,
	Development,
	Unknown,
}

impl AppVariant {
	pub fn get_current() -> Self {
		if cfg!(debug_assertions) {
			return Self::Development;
		}

		if cfg!(target_os = "windows") {
			return if has_portable_marker() {
				Self::WindowsPortable
			} else {
				Self::WindowsNsis
			};
		}

		if cfg!(target_os = "linux") {
			return if std::env::var_os("APPIMAGE").is_some() {
				Self::LinuxAppImage
			} else {
				Self::Unknown
			};
		}

		if cfg!(target_os = "macos") {
			return Self::MacosBundle;
		}

		Self::Unknown
	}
}

fn has_portable_marker() -> bool {
	let Ok(exe_path) = std::env::current_exe() else {
		log::warn!("Failed to find own executable path to detect app variant");
		return false;
	};

	has_marker(&exe_path).unwrap_or_else(|err| {
		log::warn!("Failed to read app variant marker from executable: {err}");
		false
	})
}

fn has_marker(exe_path: &Path) -> Result<bool, std::io::Error> {
	let mut file = File::open(exe_path)?;
	let length = file.metadata()?.len();
	let marker_length = u64::try_from(PORTABLE_MARKER.len()).unwrap_or(u64::MAX);

	if length < marker_length {
		return Ok(false);
	}

	file.seek(SeekFrom::End(
		-i64::try_from(marker_length).unwrap_or(i64::MAX),
	))?;

	let mut tail = vec![0u8; PORTABLE_MARKER.len()];
	file.read_exact(&mut tail)?;

	Ok(tail == PORTABLE_MARKER)
}
