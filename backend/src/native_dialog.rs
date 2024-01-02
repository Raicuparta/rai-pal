use std::ptr;

use log::error;
use winapi::um::winuser::{
	MessageBoxW,
	MB_ICONERROR,
	MB_OK,
	MB_SYSTEMMODAL,
};

fn to_wide_string_ptr(text: &str) -> Vec<u16> {
	text.encode_utf16().chain(std::iter::once(0)).collect()
}

pub fn error(message: &str) {
	error!("{message}");

	unsafe {
		MessageBoxW(
			ptr::null_mut(),
			to_wide_string_ptr(message).as_ptr(),
			to_wide_string_ptr("Rai Pal").as_ptr(),
			MB_OK | MB_ICONERROR | MB_SYSTEMMODAL,
		);
	}
}
