use std::{
	fs,
	path::{
		Path,
		PathBuf,
	},
};

use rai_pal_proc_macros::serializable_struct;

use crate::{
	game::DbGame,
	game_mods::game_mod::ModConfigDestinationType,
	remote_config,
	result::CoreResult,
};

#[serializable_struct]
pub struct ModConfig {
	pub destination_path: String,
	pub destination_type: ModConfigDestinationType,
	pub mod_id_override: Option<String>,
}

impl ModConfig {
	pub async fn download(&self, game: &DbGame, config_file: &str, overwrite: bool) -> CoreResult {
		// TODO: handle tokens.
		let destination_path = PathBuf::from(&self.destination_path);

		if config_exists(&destination_path)? {
			if overwrite {
				if destination_path.is_dir() {
					fs::remove_dir_all(&destination_path)?;
				} else {
					fs::remove_file(&destination_path)?;
				}
			} else {
				return Ok(());
			}
		}

		if let Some(parent) = destination_path.parent() {
			fs::create_dir_all(parent)?;
		}

		match self.destination_type {
			ModConfigDestinationType::File => {
				remote_config::download_config_file(config_file, game, &destination_path).await?;
			}
			ModConfigDestinationType::Folder => {
				remote_config::download_config_folder(config_file, game, &destination_path).await?;
			}
		}

		Ok(())
	}
}

fn config_exists(path: &Path) -> CoreResult<bool> {
	if !path.try_exists()? {
		return Ok(false);
	}

	if path.is_dir() && fs::read_dir(path)?.next().is_none() {
		return Ok(false);
	}

	Ok(true)
}
