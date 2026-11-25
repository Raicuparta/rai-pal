use std::path::PathBuf;

use crate::{
	game::Game,
	mods::{manifest::Mod, replacement_token::replace_tokens},
	result::{Error, Result},
};

pub struct InstalledMod {
	pub game_mod: Mod,
	pub game: Option<Game>,
	pub installed_version: Option<String>,
	pub installed_hash: Option<String>,
}

impl InstalledMod {
	pub fn new(
		game_mod: Mod,
		game: Option<Game>,
		installed_version: Option<String>,
		installed_hash: Option<String>,
	) -> Self {
		Self {
			game_mod,
			game,
			installed_version,
			installed_hash,
		}
	}

	pub async fn uninstall(&self) -> Result {
		let install = self.game_mod.install.as_ref().ok_or_else(|| {
			Error::ModInfoMissing(self.game_mod.id.clone(), "install".to_string())
		})?;

		if let Some(extract_actions) = install.extract.as_ref() {
			for extract_action in extract_actions {
				let destination_path = PathBuf::from(replace_tokens(
					&extract_action.destination,
					self.game.as_ref(),
					&self.game_mod,
				));

				if destination_path.is_dir() {
					fs_err::tokio::remove_dir_all(&destination_path).await?;
				} else if destination_path.exists() {
					fs_err::tokio::remove_file(&destination_path).await?;
				}
			}
		}

		if let Some(copy_actions) = install.copy.as_ref() {
			for copy_action in copy_actions {
				let destination_path = PathBuf::from(replace_tokens(
					&copy_action.destination,
					self.game.as_ref(),
					&self.game_mod,
				));

				if destination_path.exists() {
					fs_err::tokio::remove_file(&destination_path).await?;
				}
			}
		}

		if let Some(write_actions) = install.write.as_ref() {
			for write_action in write_actions {
				let destination_path = PathBuf::from(replace_tokens(
					&write_action.destination,
					self.game.as_ref(),
					&self.game_mod,
				));

				if destination_path.exists() {
					fs_err::tokio::remove_file(&destination_path).await?;
				}
			}
		}

		let manifest_path = self.game_mod.get_manifest_target_path(self.game.as_ref())?;
		if manifest_path.exists() {
			fs_err::tokio::remove_file(manifest_path).await?;
		}

		Ok(())
	}
}
