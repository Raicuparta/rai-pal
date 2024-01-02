use std::{
	ffi::OsStr,
	io,
	os::windows::ffi::OsStrExt,
	path::Path,
};

use winapi::um::{
	shellapi::ShellExecuteW,
	winuser::SW_SHOW,
};

use crate::{
	paths,
	Result,
};

pub fn run_as_admin(exe_path: &Path, parameters: &str) -> Result {
	let directory = path_to_wide(paths::path_parent(exe_path)?);

	let result = unsafe {
		ShellExecuteW(
			ptr::null_mut(),
			str_to_wide("runas").as_ptr(),
			path_to_wide(exe_path).as_ptr(),
			str_to_wide(parameters).as_ptr(),
			directory.as_ptr(),
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

fn os_str_to_wide(os_str: &OsStr) -> Vec<u16> {
	os_str.encode_wide().chain(std::iter::once(0)).collect()
}

fn str_to_wide(text: &str) -> Vec<u16> {
	os_str_to_wide(OsStr::new(text))
}

fn path_to_wide(path: &Path) -> Vec<u16> {
	os_str_to_wide(path.as_os_str())
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

const WEBVIEW_ERROR_MESSAGE: &str = "Webview error. This usually means something is wrong with your Webview2 installation.\n\nWould you like to attempt to repair it?";
const WEBVIEW_REPAIR_FAILED_MESSAGE: &str = "Ok, that didn't work either.\n\nWould you like to open the Microsoft website to download Webview2 yourself?";

const WEBVIEW_WEBSITE_URL: &str =
	"https://developer.microsoft.com/microsoft-edge/webview2#download";

const DEFAULT_WEBVIEW2_REPAIR_PATH: &str =
	r"C:\Program Files (x86)\Microsoft\EdgeUpdate\MicrosoftEdgeUpdate.exe";
const DEFAULT_WEBVIEW2_REPAIR_ARGS: &str =
	"/install appguid={F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}&appname=Microsoft%20Edge%20WebView&needsadmin=true&repairtype=windowsonlinerepair /installsource otherinstallcmd";

const WEBVIEW_REGISTRY_KEY: &str =
	r"SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall\Microsoft EdgeWebView";

// Even though Tauri is supposed to check if Webview2 is installed and install it if needed,
// it's easy to corrupt it in such a way that it gets detected as installed, even though it's actually unusable.
// This causes the app to silently crash on launch. So we need to give the user some options on what to do in this case.
pub fn webview_error_dialog(error_text: &str) {
	error!("{error_text}");
	if error_question_dialog(WEBVIEW_ERROR_MESSAGE) {
		let (path, command) = get_webview2_repair();

		run_as_admin(&path, &command).unwrap_or_else(|repair_error| {
			error!("{repair_error}");
			if error_question_dialog(WEBVIEW_REPAIR_FAILED_MESSAGE) {
				open::that_detached(WEBVIEW_WEBSITE_URL).unwrap_or_else(|open_website_error| {
					// If we reached here then everything failed, just give up please.
					error_dialog(&format!(
						"Somehow even that failed. Please report this error.\n\nError: {open_website_error}"
					));
				});
			}
		});
	}

	// TODO open logs folder here.
}

// Seems to be common for Webview2 to be broken, but with Edge still present,
// so the Edge installer can be used to repair Webview2.
fn get_webview2_repair() -> (PathBuf, String) {
	// We crawl through the windows registry sewage to find the command for repairing Webview2.
	RegKey::predef(HKEY_LOCAL_MACHINE)
		.open_subkey(WEBVIEW_REGISTRY_KEY)
		.and_then(|key| key.get_value::<String, _>("ModifyPath"))
		.map_or_else(
			|error| get_fallback_webview2_repair(&error.to_string()),
			|path_with_args| {
				// The command is store in the registry in a format like "C:/Some/Path/To/the.exe \and some=args".
				if let Some((_, path, args)) = regex_captures!(r#""(.+)" ?(.*)"#, &path_with_args) {
					return (PathBuf::from(path), args.to_string());
				}
				get_fallback_webview2_repair(&format!(
					"Failed to parse registry item: {path_with_args}"
				))
			},
		)
}

// If we aren't able to figure out the Webview2 repair command from the registry,
// we just fall back to a hardcoded one, which should be the case for the majority of users.
// If we reach here it's unlikely that the repair exe will be present anyway, but worth a shot I guess.
fn get_fallback_webview2_repair(message: &str) -> (PathBuf, String) {
	error!("Failed to get WebView2 repair exe path from registry. Attempting with default path. {message}");
	(
		PathBuf::from(DEFAULT_WEBVIEW2_REPAIR_PATH),
		DEFAULT_WEBVIEW2_REPAIR_ARGS.to_string(),
	)
}
