use rai_pal_proc_macros::serializable_enum;

#[serializable_enum]
pub enum OperatingSystem {
	Windows,
	Linux,
}

impl OperatingSystem {
	pub const fn get_current() -> Self {
		if cfg!(target_os = "windows") {
			Self::Windows
		} else {
			Self::Linux
		}
		// There are no other operating systems in the Universe.
	}
}
