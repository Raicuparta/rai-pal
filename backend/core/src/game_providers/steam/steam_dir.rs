use std::{
	path::{
		Path,
		PathBuf,
	},
	time::{
		SystemTime,
		UNIX_EPOCH,
	},
};

use steamlocate::SteamDir;

use crate::result::{
	Error,
	Result,
};

pub fn get_appinfo_path(steam_path: &Path) -> PathBuf {
	steam_path.join("appcache/appinfo.vdf")
}

fn get_modified_time(path: &Path) -> SystemTime {
	std::fs::metadata(path)
		.and_then(|metadata| metadata.modified())
		.unwrap_or(UNIX_EPOCH)
}

// Multiple Steam installations can exist. Making Rai Pal support multiple at once
// is a bit of a big refactor, so instead I'm just using some heuristics to pic one.
pub fn find_steam_dir() -> Result<SteamDir> {
	let steam_dirs = steamlocate::locate_all()?;


	let prefer_appinfo = steam_dirs
		.iter()
		.any(|steam_dir| get_appinfo_path(steam_dir.path()).is_file());

	let mut best: Option<((SystemTime, SystemTime, SystemTime, u64), SteamDir)> = None;

	for steam_dir in steam_dirs {
		let steam_path = steam_dir.path();
		let appinfo_path = get_appinfo_path(steam_path);

		if prefer_appinfo && !appinfo_path.is_file() {
			continue;
		}

		let appinfo_size = std::fs::metadata(&appinfo_path).map_or(0, |metadata| metadata.len());

		let score = (
			get_modified_time(&steam_path.join("config/config.vdf")),
			get_modified_time(&steam_path.join("config/loginusers.vdf")),
			get_modified_time(&appinfo_path),
			appinfo_size,
		);

		if best
			.as_ref()
			.is_none_or(|(best_score, _)| score > *best_score)
		{
			best = Some((score, steam_dir));
		}
	}

	best.map(|(_, steam_dir)| steam_dir)
		.ok_or(Error::SteamDirNotFound())
}
