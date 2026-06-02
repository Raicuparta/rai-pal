use super::mod_provider::ModProvider;
use crate::{
	app_paths,
	local_database::{
		game_database::DbMutex,
		mod_database::ModDatabase,
	},
	mods::game_mod::{
		GameMod,
		ModDownload,
	},
	path_extensions::{
		AsValidStr,
		PathExt,
	},
	result::{
		Error,
		Result,
	},
};

pub struct FolderModProvider {
	pub folder_path: std::path::PathBuf,
}

impl ModProvider for FolderModProvider {
	fn default() -> Result<Self> {
		Ok(Self {
			folder_path: app_paths::local_mods_path()?,
		})
	}

	async fn insert_mods(&self, db: &DbMutex) -> Result {
		for manifest_path in self.folder_path.join("*").join(GameMod::FILE_NAME).glob() {
			let Some(mut game_mod) = GameMod::from_file(&manifest_path) else {
				continue;
			};

			if game_mod.download.is_some() {
				return Err(Error::LocalModCantDefineDownload(game_mod.id));
			}

			game_mod.download = Some(ModDownload {
				id: "0.0.0".to_string(),
				url: format!("file://{}", manifest_path.try_parent()?.try_to_str()?),
			});

			db.insert_mod(&game_mod);
		}

		Ok(())
	}
}
