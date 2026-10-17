use fs_err::PathExt;
use std::path::{Path, PathBuf};

use rai_pal_proc_macros::serializable_struct;

use crate::{
	game::DbGame,
	mods::{
		game_mod::{GameMod, ModConfigDestinationType},
		replacement_token::replace_tokens,
	},
	remote_config::{self, RemoteConfig},
	result::Result,
};

#[serializable_struct]
pub struct ModConfig {
	pub destination_path: String,
	pub destination_type: ModConfigDestinationType,
	pub mod_id_override: Option<String>,
}

impl ModConfig {
	pub async fn download(
		&self,
		game: &DbGame,
		game_mod: &GameMod,
		remote_config: &RemoteConfig,
		overwrite: bool,
	) -> Result {
		let destination_path =
			PathBuf::from(replace_tokens(&self.destination_path, Some(game), game_mod));

		if config_exists(&destination_path).await? {
			if overwrite {
				if destination_path.is_dir() {
					fs_err::tokio::remove_dir_all(&destination_path).await?;
				} else {
					fs_err::tokio::remove_file(&destination_path).await?;
				}
			} else {
				return Ok(());
			}
		}

		if let Some(parent) = destination_path.parent() {
			fs_err::tokio::create_dir_all(parent).await?;
		}

		match self.destination_type {
			ModConfigDestinationType::File => {
				remote_config::download_config_file(&remote_config.file, game, &destination_path)
					.await?;
			}
			ModConfigDestinationType::Folder => {
				remote_config::download_config_folder(&remote_config.file, game, &destination_path)
					.await?;
			}
		}

		Ok(())
	}
}

async fn config_exists(path: &Path) -> Result<bool> {
	if !path.fs_err_try_exists()? {
		return Ok(false);
	}

	if path.is_dir()
		&& fs_err::tokio::read_dir(path)
			.await?
			.next_entry()
			.await?
			.is_none()
	{
		return Ok(false);
	}

	Ok(true)
}
