use std::{
	path::PathBuf,
	ptr,
};

use lazy_regex::regex_captures;
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
use winreg::{
	enums::HKEY_LOCAL_MACHINE,
	RegKey,
};

use crate::windows::run_as_admin;

// TODO use OsStr encode_wide
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
const DEFAULT_WEBVIEW2_REPAIR_PATH: &str =
	"C:\\Program Files (x86)\\Microsoft\\EdgeUpdate\\MicrosoftEdgeUpdate.exe";
const DEFAULT_WEBVIEW2_REPAIR_ARGS: &str = "/install appguid={F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}&appname=Microsoft%20Edge%20WebView&needsadmin=true&repairtype=windowsonlinerepair /installsource otherinstallcmd";

fn get_fallback_webview2_repair(message: &str) -> (PathBuf, String) {
	error!("Failed to get WebView2 repair exe path from registry. Attempting with default path. {message}");
	(
		PathBuf::from(DEFAULT_WEBVIEW2_REPAIR_PATH),
		DEFAULT_WEBVIEW2_REPAIR_ARGS.to_string(),
	)
}

fn get_webview2_repair() -> (PathBuf, String) {
	RegKey::predef(HKEY_LOCAL_MACHINE).open_subkey("SOFTWARE\\WOW6432Node\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\Microsoft EdgeWebView")
	.and_then(|key| key.get_value::<String, _>("ModifyPath")).map_or_else(|error| {
		get_fallback_webview2_repair(&error.to_string())
	}, |path_with_args| {
		if let Some((_, path, args)) = regex_captures!(r#""(.+)" ?(.*)"#, &path_with_args) {
			return (PathBuf::from(path), args.to_string());
		}
		get_fallback_webview2_repair(&format!("Failed to parse registry item: {path_with_args}"))
	})
}

pub fn webview_error_dialog(error_text: &str) {
	error!("{error_text}");
	if error_question_dialog(WEBVIEW_ERROR_MESSAGE) {
		let (path, command) = get_webview2_repair();

		run_as_admin(&path, &command).unwrap_or_else(|repair_error| {
			error!("{repair_error}");
			if error_question_dialog(WEBVIEW_REPAIR_FAILED_MESSAGE) {
				open::that_detached(WEBVIEW_WEBSITE_URL).unwrap_or_else(|open_website_error| {
					error_dialog(&format!(
						"Somehow even that failed. Error: {open_website_error}"
					));
				});
			}
		});
	}

	// TODO open logs folder here.
}
