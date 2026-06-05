use std::{
	ffi::OsStr,
	process::Stdio,
};

use crate::{
	path_extensions::AsValidStr,
	result::{
		Error,
		Result,
	},
};

// Weird workaround for AppImage builds.
pub fn open_detached_better(path: impl AsRef<OsStr>) -> Result {
	let mut last_error = Error::NoCommandForOpen(path.as_ref().try_to_str()?.to_string());

	for mut cmd in open::commands(path) {
		cmd.env_remove("LD_LIBRARY_PATH");
		cmd.env_remove("QT_PLUGIN_PATH");
		cmd.env_remove("APPDIR");
		cmd.env_remove("APPIMAGE");

		cmd.stdin(Stdio::null())
			.stdout(Stdio::null())
			.stderr(Stdio::null());

		match cmd.spawn() {
			Ok(_) => return Ok(()),
			Err(e) => last_error = e.into(),
		}
	}

	Err(last_error)
}
