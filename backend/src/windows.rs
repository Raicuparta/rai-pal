use std::path::Path;

use crate::Result;

#[cfg(windows)]
pub fn run_as_admin(exe_path: &Path, parameters: &str) -> Result {
	use std::{
		ffi::OsStr,
		io,
		os::windows::ffi::OsStrExt,
		ptr,
	};

	use winapi::{
		ctypes::c_int,
		um::{
			shellapi::ShellExecuteW,
			winuser::SW_SHOW,
		},
	};

	use crate::paths;

	let exe_path_str: Vec<u16> = OsStr::new(&exe_path).encode_wide().chain(Some(0)).collect();

	let verb = OsStr::new("runas");
	let verb_str: Vec<u16> = verb.encode_wide().chain(Some(0)).collect();

	let parameters_str: Vec<u16> = OsStr::new(&parameters)
		.encode_wide()
		.chain(Some(0))
		.collect();

	let directory = paths::path_parent(exe_path)?;
	let directory_str: Vec<u16> = OsStr::new(directory).encode_wide().chain(Some(0)).collect();

	let result = unsafe {
		ShellExecuteW(
			ptr::null_mut(),
			verb_str.as_ptr(),
			exe_path_str.as_ptr(),
			parameters_str.as_ptr(),
			directory_str.as_ptr(),
			SW_SHOW,
		)
	};

	#[allow(clippy::as_conversions)]
	if result as c_int > 32 {
		Ok(())
	} else {
		Err(io::Error::last_os_error())?
	}
}

#[cfg(not(windows))]
pub const fn run_as_admin(_exe_path: &Path, _parameters: &str) -> Result {
	use crate::result::Error;

	Err(Error::NotImplemented)
}
