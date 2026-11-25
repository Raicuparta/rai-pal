use std::{
	path::{Path, PathBuf},
	time::{SystemTime, UNIX_EPOCH},
};

use steamlocate::SteamDir;

use crate::result::{ContextResult, Error, LogErrExt, Result};

pub fn get_appinfo_path(steam_path: &Path) -> PathBuf {
	steam_path.join("appcache/appinfo.vdf")
}

pub fn get_appinfo_dir(steam_path: &Path) -> PathBuf {
	steam_path.join("appcache/appinfo")
}

pub fn get_appinfo_paths(steam_path: &Path) -> Result<Vec<PathBuf>> {
	let appinfo_dir = get_appinfo_dir(steam_path);

	if appinfo_dir.is_dir() {
		let mut paths = Vec::new();

		for entry in fs_err::read_dir(appinfo_dir)? {
			let path = entry?.path();
			let Some(file_name) = path.file_name().and_then(|file_name| file_name.to_str()) else {
				continue;
			};

			if file_name.starts_with("appinfo_")
				&& Path::new(file_name)
					.extension()
					.is_some_and(|extension| extension.eq_ignore_ascii_case("vdf"))
			{
				paths.push(path);
			}
		}

		if !paths.is_empty() {
			paths.sort();
			return Ok(paths);
		}
	}

	let appinfo_path = get_appinfo_path(steam_path);
	if appinfo_path.is_file() {
		Ok(vec![appinfo_path])
	} else {
		Ok(Vec::new())
	}
}

fn has_appinfo(steam_path: &Path) -> bool {
	get_appinfo_dir(steam_path).is_dir() || get_appinfo_path(steam_path).is_file()
}

fn get_appinfo_cache_path(steam_path: &Path) -> PathBuf {
	let appinfo_dir = get_appinfo_dir(steam_path);
	if appinfo_dir.is_dir() {
		appinfo_dir
	} else {
		get_appinfo_path(steam_path)
	}
}

fn get_modified_time(path: &Path) -> SystemTime {
	fs_err::metadata(path)
		.and_then(|metadata| metadata.modified())
		.ok_or_log(&format!(
			"Failed to get modification time for {}",
			path.display()
		))
		.unwrap_or(UNIX_EPOCH)
}

// Multiple Steam installations can exist. Making Rai Pal support multiple at once
// is a bit of a big refactor, so instead I'm just using some heuristics to pic one.
pub fn find_steam_dir() -> Result<SteamDir> {
	let steam_dirs = steamlocate::locate_all().context("failed to locate Steam installations")?;

	let prefer_appinfo = steam_dirs
		.iter()
		.any(|steam_dir| has_appinfo(steam_dir.path()));

	let mut best: Option<((SystemTime, SystemTime, SystemTime, u64), SteamDir)> = None;

	for steam_dir in steam_dirs {
		let steam_path = steam_dir.path();
		if prefer_appinfo && !has_appinfo(steam_path) {
			continue;
		}

		let appinfo_cache_path = get_appinfo_cache_path(steam_path);
		let appinfo_size =
			fs_err::metadata(&appinfo_cache_path).map_or(0, |metadata| metadata.len());

		let score = (
			get_modified_time(&steam_path.join("config/config.vdf")),
			get_modified_time(&steam_path.join("config/loginusers.vdf")),
			get_modified_time(&appinfo_cache_path),
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
