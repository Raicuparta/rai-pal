use std::{
	path::PathBuf,
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

use crate::windows::run_as_admin;

fn to_wide_string_ptr(text: &str) -> Vec<u16> {
	text.encode_utf16().chain(std::iter::once(0)).collect()
}

fn base_error_dialog(error_text: &str, flags: c_uint) -> c_int {
	error!("{error_text}");

	unsafe {
		MessageBoxW(
			ptr::null_mut(),
			to_wide_string_ptr(error_text).as_ptr(),
			to_wide_string_ptr("Rai Pal").as_ptr(),
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

const WEBVIEW_ERROR_MESSAGE: &str = "Webview error. This usually means something is wrong with your Webview2 installation.\n\nWould you like to attempt to repair it?";
const WEBVIEW_REPAIR_FAILED_MESSAGE: &str = "Ok, that didn't work either.\n\nWould you like to open the Microsoft website to download Webview2 yourself?";
const WEBVIEW_WEBSITE_URL: &str =
	"https://developer.microsoft.com/microsoft-edge/webview2#download";

pub fn webview_error_dialog(error_text: &str) {
	error!("{error_text}");
	if error_question_dialog(WEBVIEW_ERROR_MESSAGE) {
		run_as_admin(
			&PathBuf::from("C:\\Program Files (x86)\\Microsoft\\EdgeUpdate\\MicrosoftEdgeUpdate.exe"),
			"/install appguid={F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}&appname=Microsoft%20Edge%20WebView&needsadmin=true&repairtype=windowsonlinerepair /installsource otherinstallcmd")
			.unwrap_or_else(|repair_error| {
				error!("{repair_error}");
				if error_question_dialog(WEBVIEW_REPAIR_FAILED_MESSAGE) {
					open::that_detached(WEBVIEW_WEBSITE_URL).unwrap_or_else(|open_website_error| {
						error_dialog(&format!("Somehow even that failed. Error: {open_website_error}"));
					});
				}
		});
	}

	// TODO open logs folder here.
}
