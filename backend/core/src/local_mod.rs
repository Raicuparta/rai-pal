use std::{
	collections::HashMap,
	fs,
	path::{
		Path,
		PathBuf,
	},
};

use crate::{
	game::DbGame,
	game_mods::{
		mod_config::ModConfig,
		mod_database::GameMod,
	},
	paths::{
		self,
		open_folder_or_parent,
	},
	result::Result,
};

impl GameMod {
	pub fn open_folder(&self) -> Result {
		open_folder_or_parent(&self.get_local_folder_path()?)
	}

	pub async fn install(&self, game: &DbGame) -> Result {
		todo!();
	}

	pub async fn run(&self, game: &DbGame) -> Result {
		todo!();
	}

	pub async fn uninstall(&self, game: &DbGame) -> Result {
		todo!();
	}

	pub async fn run_without_game(&self) -> Result {
		todo!();
	}

	pub fn update_installed_mod_manifest(&self, game: &DbGame) -> Result {
		// TODO: make sure it doesn't happen for runnables.
		let manifest_path = game.get_installed_mod_manifest_path(&self.id)?;
		fs::create_dir_all(paths::path_parent(&manifest_path)?)?;
		let manifest_contents = serde_json::to_string_pretty(&self)?;
		fs::write(manifest_path, manifest_contents)?;

		Ok(())
	}

	pub fn get_config_path(&self, config: &ModConfig, _game: &DbGame) -> Result<PathBuf> {
		// TODO: handle tokens and game.
		Ok(PathBuf::from(&config.destination_path))
	}

	pub fn configure_mod(&self, game: &DbGame, open_folder: bool) -> Result {
		if let Some(config) = self.config.as_ref() {
			let config_path = self.get_config_path(config, game)?;
			if open_folder {
				paths::open_folder_or_parent(&config_path)?;
			} else {
				open::that_detached(config_path)?;
			}
		}

		Ok(())
	}

	pub fn open_installed_mod_folder(&self, _game: &DbGame) -> Result {
		todo!();
	}

	pub fn delete_local(&self) -> Result {
		let path = self.get_local_folder_path()?;
		if path.exists() {
			fs::remove_dir_all(&path)?;
		}

		Ok(())
	}

	pub fn get_local_folder_path(&self) -> Result<PathBuf> {
		Ok(paths::local_mods_path()?.join(&self.id))
	}

	pub fn get_local_manifest_path(&self) -> Result<PathBuf> {
		Ok(get_manifest_path(&self.get_local_folder_path()?))
	}
}

pub fn get_all() -> Result<HashMap<String, GameMod>> {
	Ok(
		paths::glob_path(&paths::local_mods_path()?.join("*").join(GameMod::FILE_NAME))
			.iter()
			.filter_map(|manifest_path| {
				GameMod::from_file(manifest_path).map(|local_mod| (local_mod.id.clone(), local_mod))
			})
			.collect(),
	)
}

pub fn get_manifest_path(target_path: &Path) -> PathBuf {
	target_path.join(GameMod::FILE_NAME)
}
