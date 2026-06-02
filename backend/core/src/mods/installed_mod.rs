use std::{
	fs,
	path::PathBuf,
};

use crate::{
	game::DbGame,
	mods::{
		game_mod::GameMod,
		mod_config::ModConfig,
		replacement_token::replace_tokens,
	},
	open_better::open_detached_better,
	path_extensions::PathExt,
	result::{
		Error,
		Result,
	},
};

pub struct InstalledMod {
	pub game_mod: GameMod,
	pub game: DbGame,
}

impl InstalledMod {
	pub const fn new(game_mod: GameMod, game: DbGame) -> Self {
		Self { game_mod, game }
	}

	pub fn open_folder(&self) -> Result {
		PathBuf::from(replace_tokens(
			self.game_mod
				.get_install()?
				.main_installed_folder_path
				.as_ref()
				.ok_or_else(|| {
					Error::ModInfoMissing(
						self.game_mod.id.clone(),
						"main_installed_folder_path".to_string(),
					)
				})?,
			&self.game,
			&self.game_mod,
		))
		.open_folder_or_parent()?;

		Ok(())
	}

	pub fn uninstall(&self) -> Result {
		let install = self.game_mod.install.as_ref().ok_or_else(|| {
			Error::ModInfoMissing(self.game_mod.id.clone(), "install".to_string())
		})?;

		if let Some(extract_actions) = install.extract.as_ref() {
			for extract_action in extract_actions {
				let destination_path = PathBuf::from(replace_tokens(
					&extract_action.destination,
					&self.game,
					&self.game_mod,
				));

				if destination_path.is_dir() {
					destination_path.remove_if_exists()?;
				} else if destination_path.exists() {
					fs::remove_file(&destination_path)?;
				}
			}
		}

		if let Some(write_actions) = install.write.as_ref() {
			for write_action in write_actions {
				let destination_path = PathBuf::from(replace_tokens(
					&write_action.destination,
					&self.game,
					&self.game_mod,
				));

				if destination_path.exists() {
					fs::remove_file(&destination_path)?;
				}
			}
		}

		let manifest_path = self
			.game
			.get_installed_mod_manifest_path(&self.game_mod.id)?;
		if manifest_path.exists() {
			fs::remove_file(manifest_path)?;
		}

		Ok(())
	}

	fn get_config_path(&self, config: &ModConfig) -> PathBuf {
		PathBuf::from(&replace_tokens(
			&config.destination_path,
			&self.game,
			&self.game_mod,
		))
	}

	pub fn configure(&self, open_folder: bool) -> Result {
		if let Some(config) = self.game_mod.config.as_ref() {
			let config_path = self.get_config_path(config);
			if open_folder {
				config_path.open_folder_or_parent()?;
			} else {
				open_detached_better(config_path)?;
			}
		}

		Ok(())
	}
}
