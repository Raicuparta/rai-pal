use std::{
	ffi::OsStr,
	os::windows::ffi::OsStrExt,
	ptr,
};

use log::error;
use winapi::{
	ctypes::{
		c_int,
		c_uint,
	},
	um::winuser::{
		MessageBoxW,
		IDYES,
		MB_ICONERROR,
		MB_OK,
		MB_SYSTEMMODAL,
		MB_YESNO,
	},
};

use crate::paths;

fn os_str_to_wide(os_str: &OsStr) -> Vec<u16> {
	os_str.encode_wide().chain(std::iter::once(0)).collect()
}

fn str_to_wide(text: &str) -> Vec<u16> {
	os_str_to_wide(OsStr::new(text))
}

fn base_error_dialog(error_text: &str, flags: c_uint) -> c_int {
	error!("{error_text}");

	unsafe {
		MessageBoxW(
			ptr::null_mut(),
			str_to_wide(error_text).as_ptr(),
			str_to_wide("Rai Pal").as_ptr(),
			flags | MB_ICONERROR | MB_SYSTEMMODAL,
		)
	}
}

pub fn error_dialog(error_text: &str) {
	base_error_dialog(error_text, MB_OK);
}

pub fn error_question_dialog(error_text: &str) -> bool {
	base_error_dialog(error_text, MB_YESNO) == IDYES
}

const WEBVIEW_ERROR_MESSAGE: &str = r#"Webview error. This usually means something is wrong with your Webview2 installation.

You can try to repair it by going to Windows Settings > Apps > Installed Apps, then finding "Microsoft Edge WebView2 Runtime" and picking "Modify" from the three dots menu.

If that doesn't work, search online for "Microsoft Edge WebView2", and download the installer from Microsoft's website.

Would you like to open the logs folder?"#;

fn try_open_logs_folder() {
	if let Err(error) = paths::open_logs_folder() {
		error!("Failed to even open the logs folder: {error}");
	}
}

// Even though Tauri is supposed to check if Webview2 is installed and install it if needed,
// it's easy to corrupt it in such a way that it gets detected as installed, even though it's actually unusable.
// This causes the app to silently crash on launch. So we need to give the user some options on what to do in this case.
pub fn webview_error_dialog(error_text: &str) {
	error!("{error_text}");
	if error_question_dialog(WEBVIEW_ERROR_MESSAGE) {
		try_open_logs_folder();
	}
}
