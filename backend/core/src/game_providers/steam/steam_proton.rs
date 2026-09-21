use std::{
	collections::BTreeMap,
	fs,
	path::{Path, PathBuf},
};

use steamlocate::Library;

use crate::{
	game::DbGame,
	game_launch::GameLaunch,
	game_providers::{
		game_provider::{self, WineProviderActions},
		steam::{
			steam_dir::find_steam_dir,
			steam_provider::Steam,
			steam_runtime::{
				get_native_runtime_command, wrap_game_with_steam_runtime, wrap_with_steam_runtime,
			},
		},
	},
	path_extensions::{AsValidStr, PathExt},
	result::{Error, Result},
	wine,
};

impl WineProviderActions for Steam {
	fn get_wine_prefix_path(&self, game: &DbGame) -> Result<PathBuf> {
		let app_id: u32 = game.external_id.parse()?;
		let steam_dir = find_steam_dir()?;

		if let Some((_, library)) = steam_dir.find_app(app_id)? {
			return Ok(get_prefix_path(&library, &game.external_id));
		}

		for library in (steam_dir.libraries()?).flatten() {
			let prefix_path = get_prefix_path(&library, &game.external_id);

			if prefix_path.exists() {
				return Ok(prefix_path);
			}
		}

		Err(Error::SteamProton(format!(
			"Library not found for Steam app {app_id} and no compatdata prefix exists in any Steam library"
		)))
	}

	fn get_wine_binary_path(&self, game: &DbGame) -> Result<PathBuf> {
		let prefix_path = self.get_wine_prefix_path(game)?;
		let compat_data_path = prefix_path.try_parent()?;

		Ok(get_proton_dir(compat_data_path)?
			.join("files")
			.join("bin")
			.join("wine"))
	}

	fn get_run_with_wine_command(&self, game: &DbGame) -> Result<GameLaunch> {
		let wine_prefix_path = self.get_wine_prefix_path(game)?;
		let compat_data_path = wine_prefix_path.try_parent()?;
		let wine_binary_path = self.get_wine_binary_path(game)?;

		build_wine_command(
			game,
			&wine_prefix_path,
			compat_data_path,
			&wine_binary_path,
			false,
		)
	}

	fn get_game_run_with_wine_command(&self, game: &DbGame) -> Result<GameLaunch> {
		let wine_prefix_path = self.get_wine_prefix_path(game)?;
		let compat_data_path = wine_prefix_path.try_parent()?;
		let wine_binary_path = self.get_wine_binary_path(game)?;

		build_wine_command(
			game,
			&wine_prefix_path,
			compat_data_path,
			&wine_binary_path,
			true,
		)
	}

	fn get_native_run_environment(&self, game: &DbGame) -> Result<BTreeMap<String, String>> {
		let mut environment = get_steam_launch_environment(game)?;
		game_provider::extend_native_run_environment(&mut environment);
		Ok(environment)
	}

	fn get_native_run_command(
		&self,
		game: &DbGame,
		exe_path: &Path,
		args: &[String],
	) -> Result<Option<GameLaunch>> {
		Ok(get_native_runtime_command(
			&game.external_id,
			exe_path,
			args,
		))
	}

	fn set_wine_dll_overrides(&self, game: &DbGame, dll_overrides: &[String]) -> Result {
		let prefix_path = self.get_wine_prefix_path(game)?;
		wine::set_wine_dll_overrides_in_reg(&prefix_path, dll_overrides)
			.map_err(|err| Error::SteamProton(err.to_string()))
	}
}

fn build_wine_command(
	game: &DbGame,
	wine_prefix_path: &Path,
	compat_data_path: &Path,
	wine_binary_path: &Path,
	launch_game: bool,
) -> Result<GameLaunch> {
	let mut launch = if launch_game {
		wrap_game_with_steam_runtime(&game.external_id, compat_data_path, wine_binary_path)
	} else {
		wrap_with_steam_runtime(compat_data_path, wine_binary_path)
	};

	launch
		.env("WINEPREFIX", wine_prefix_path)
		.env("STEAM_COMPAT_DATA_PATH", compat_data_path)
		.env("WINEFSYNC", "1")
		.envs(&get_steam_launch_environment(game)?);

	Ok(launch)
}

fn get_steam_launch_environment(game: &DbGame) -> Result<BTreeMap<String, String>> {
	let mut env = BTreeMap::new();

	for key in [
		"SteamAppId",
		"SteamGameId",
		"SteamOverlayGameId",
		"STEAM_COMPAT_APP_ID",
	] {
		env.insert(key.to_string(), game.external_id.clone());
	}

	env.insert("SteamClientLaunch".to_string(), "1".to_string());
	env.insert("SteamEnv".to_string(), "1".to_string());

	if let Some(parent) = game.exe_path.as_ref().and_then(|exe| exe.parent()) {
		env.insert(
			"STEAM_COMPAT_INSTALL_PATH".to_string(),
			parent.try_to_str()?.to_string(),
		);
	}

	Ok(env)
}

pub fn get_proton_dir(compat_data_path: &Path) -> Result<PathBuf> {
	let config_info_data = fs::read_to_string(compat_data_path.join("config_info"))?;

	let proton_lib_path_line = match config_info_data.lines().nth(2) {
		Some(line) if !line.trim().is_empty() => line.trim(),
		_ => {
			return Err(Error::SteamProton(
				"Steam Proton config_info is missing a valid third line".to_string(),
			));
		}
	};

	Ok(Path::new(proton_lib_path_line)
		.try_parent()?
		.try_parent()?
		.to_path_buf())
}

fn get_prefix_path(library: &Library, app_id: &str) -> PathBuf {
	library
		.path()
		.join("steamapps")
		.join("compatdata")
		.join(app_id)
		.join("pfx")
}
