use std::{
	fs,
	path::PathBuf,
};

use anyhow::Context;

use crate::{
	game::DbGame,
	game_mods::{
		game_mod::GameMod,
		mod_config::ModConfig,
		replacement_token::replace_tokens,
	},
	paths,
	result::{
		Error,
		Result,
	},
};

pub struct InstalledMod<'a> {
	pub game_mod: GameMod,
	pub game: &'a DbGame,
}

impl<'a> InstalledMod<'a> {
	pub const fn new(game_mod: GameMod, game: &'a DbGame) -> Self {
		Self { game_mod, game }
	}

	pub fn open_folder(&self) -> Result {
		paths::open_folder_or_parent(&PathBuf::from(replace_tokens(
			self.game_mod
				.get_install()?
				.main_installed_folder_path
				.as_ref()
				.with_context(|| {
					Error::ModInfoMissing(
						self.game_mod.id.clone(),
						"main_installed_folder_path".to_string(),
					)
				})?,
			self.game,
			&self.game_mod,
		)))?;

		Ok(())
	}

	pub fn uninstall(&self) -> Result {
		let install = self.game_mod.install.as_ref().with_context(|| {
			Error::ModInfoMissing(self.game_mod.id.clone(), "install".to_string())
		})?;

		let local_mod_path = self.game_mod.get_local_folder_path()?;

		if let Some(extract_actions) = install.extract.as_ref() {
			for extract_action in extract_actions {
				let source_path = local_mod_path.join(&extract_action.source);
				let destination_path = PathBuf::from(replace_tokens(
					&extract_action.destination,
					self.game,
					&self.game_mod,
				));

				if source_path.is_dir() {
					paths::remove_path_if_exists(&destination_path)?;
				} else if destination_path.exists() {
					fs::remove_file(&destination_path)?;
				}
			}
		}

		if let Some(write_actions) = install.write.as_ref() {
			for write_action in write_actions {
				let destination_path = PathBuf::from(replace_tokens(
					&write_action.destination,
					self.game,
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
			self.game,
			&self.game_mod,
		))
	}

	pub fn configure(&self, open_folder: bool) -> Result {
		if let Some(config) = self.game_mod.config.as_ref() {
			let config_path = self.get_config_path(config);
			if open_folder {
				paths::open_folder_or_parent(&config_path)?;
			} else {
				open::that_detached(config_path)?;
			}
		}

		Ok(())
	}
}
