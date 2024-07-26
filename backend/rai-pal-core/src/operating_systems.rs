use std::env;

use crate::game_executable::OperatingSystem;

pub fn get_current_os() -> OperatingSystem {
	if env::consts::OS == "windows" {
		OperatingSystem::Windows
	} else {
		OperatingSystem::Linux
	}
	// There are no other operating systems in the universe.
}
