#![cfg(target_os = "linux")]

use std::path::{Path, PathBuf};

use crate::{
	game_launch::GameLaunch,
	game_providers::steam::{steam_dir::find_steam_dir, steam_proton::get_proton_dir},
	result::LogErrExt,
};

const RUNTIME_APP_IDS: [u32; 4] = [4_183_110, 1_628_350, 1_391_110, 1_070_560];

pub fn find_steam_run() -> Option<PathBuf> {
	let path_var = std::env::var_os("PATH")?;
	std::env::split_paths(&path_var)
		.map(|dir| dir.join("steam-run"))
		.find(|candidate| candidate.exists())
}

pub fn wrap_with_steam_run(program: &Path) -> GameLaunch {
	find_steam_run().map_or_else(
		|| GameLaunch::new(program),
		|steam_run| {
			log::info!(
				"Bootstrapping Steam environment through `{}`",
				steam_run.display()
			);
			let mut launch = GameLaunch::new(steam_run);
			launch.arg(program);
			launch
		},
	)
}

fn find_reaper() -> Option<PathBuf> {
	let steam_dir = find_steam_dir().ok_or_log("Failed to find Steam directory")?;
	let reaper = steam_dir.path().join("ubuntu12_32").join("reaper");
	reaper.exists().then_some(reaper)
}

fn find_steam_launch_wrapper() -> Option<PathBuf> {
	let steam_dir = find_steam_dir().ok_or_log("Failed to find Steam directory")?;

	[
		steam_dir.path().join("ubuntu12_32"),
		steam_dir.path().join("steamrt64"),
	]
	.into_iter()
	.map(|dir| dir.join("steam-launch-wrapper"))
	.find(|candidate| candidate.exists())
}

/// Mirrors Steam's own game launch chain: runs the game under `reaper`, so Steam
/// can clean up the whole process tree, and through `steam-launch-wrapper` so it
/// gets the same environment Steam gives its games.
fn wrap_with_steam_launch(app_id: &str, program: &Path) -> GameLaunch {
	let Some(wrapper) = find_steam_launch_wrapper() else {
		return wrap_with_steam_run(program);
	};

	let mut launch = wrap_with_steam_run(&wrapper);
	launch.arg("--");

	if let Some(reaper) = find_reaper() {
		launch
			.arg(reaper)
			.arg("SteamLaunch")
			.arg(format!("AppId={app_id}"))
			.arg("--");
	}

	launch.arg(program);

	launch
}

fn find_app_dir(app_id: u32) -> Option<PathBuf> {
	let steam_dir = find_steam_dir().ok_or_log("Failed to find Steam directory")?;

	for library in (steam_dir
		.libraries()
		.ok_or_log("Failed to list Steam libraries")?)
	.flatten()
	{
		for app in library.apps().flatten() {
			if app.app_id == app_id {
				return Some(library.resolve_app_dir(&app));
			}
		}
	}

	None
}

fn find_runtime_entry_point(preferred_app_id: Option<u32>) -> Option<PathBuf> {
	let mut app_ids = Vec::new();

	if let Some(app_id) = preferred_app_id {
		app_ids.push(app_id);
	}
	app_ids.extend(RUNTIME_APP_IDS);

	for app_id in app_ids {
		let Some(dir) = find_app_dir(app_id) else {
			continue;
		};

		let entry_point = dir.join("_v2-entry-point");
		if entry_point.exists() {
			log::info!(
				"Using Steam Linux Runtime app {app_id} at `{}`",
				entry_point.display()
			);
			return Some(entry_point);
		}
	}

	None
}

fn get_proton_runtime_app_id(proton_dir: &Path) -> Option<u32> {
	let manifest = std::fs::read_to_string(proton_dir.join("toolmanifest.vdf"))
		.ok_or_log("Failed to read Proton tool manifest")?;
	let line = manifest
		.lines()
		.find(|line| line.contains("require_tool_appid"))?;

	for part in line.split('"') {
		if let Ok(app_id) = part.trim().parse::<u32>() {
			return Some(app_id);
		}
	}

	None
}

fn runtime_entry_point_for_compat_data(compat_data_path: &Path) -> Option<PathBuf> {
	let preferred_app_id = get_proton_dir(compat_data_path)
		.ok_or_log("Failed to find Proton directory")
		.and_then(|proton_dir| get_proton_runtime_app_id(&proton_dir));

	find_runtime_entry_point(preferred_app_id)
}

pub fn get_native_runtime_command(
	app_id: &str,
	program: &Path,
	args: &[String],
) -> Option<GameLaunch> {
	let entry_point = find_runtime_entry_point(None)?;

	let mut launch = wrap_with_steam_launch(app_id, &entry_point);
	launch
		.arg("--verb=waitforexitandrun")
		.arg("--")
		.arg(program)
		.args(args);

	Some(launch)
}

pub fn wrap_with_steam_runtime(compat_data_path: &Path, program: &Path) -> GameLaunch {
	build_runtime_command(compat_data_path, program, wrap_with_steam_run)
}

pub fn wrap_game_with_steam_runtime(
	app_id: &str,
	compat_data_path: &Path,
	program: &Path,
) -> GameLaunch {
	build_runtime_command(compat_data_path, program, |entry_point| {
		wrap_with_steam_launch(app_id, entry_point)
	})
}

fn build_runtime_command(
	compat_data_path: &Path,
	program: &Path,
	wrap: impl Fn(&Path) -> GameLaunch,
) -> GameLaunch {
	match runtime_entry_point_for_compat_data(compat_data_path) {
		Some(entry_point) => {
			let mut launch = wrap(&entry_point);
			launch
				.arg("--verb=waitforexitandrun")
				.arg("--")
				.arg(program);
			launch
		}
		None => wrap(program),
	}
}
