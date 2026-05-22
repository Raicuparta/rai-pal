#![cfg(target_os = "linux")]

use std::fs;

use crate::{
	paths,
	result::CoreResult,
};

// TODO: make this be a mod action?
pub fn set_up_global_wine_overrides() -> CoreResult {
	let path = paths::base_dirs()?.config_dir().join("environment.d");

	fs::create_dir_all(&path)?;

	fs::write(
		path.join("90-rai-pal-wine-overrides.conf"),
		"WINEDLLOVERRIDES=\"winhttp.dll=n,b\"",
	)?;

	Ok(())
}
