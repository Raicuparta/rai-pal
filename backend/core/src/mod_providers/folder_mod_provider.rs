use std::{
	hash::{
		DefaultHasher,
		Hash,
		Hasher,
	},
	path::PathBuf,
};

use super::mod_provider::ModProvider;
use crate::{
	app_paths,
	local_database::{
		app_database::DbMutex,
		mod_database::ModDatabase,
	},
	mod_providers::mod_provider::ModProviderId,
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
	pub folder_path: PathBuf,
}

impl FolderModProvider {
	fn compute_source_hash(&self) -> String {
		let canonical =
			std::fs::canonicalize(&self.folder_path).unwrap_or_else(|_| self.folder_path.clone());
		let mut hasher = DefaultHasher::new();
		canonical.to_string_lossy().hash(&mut hasher);
		hasher.finish().to_string()
	}
}

impl ModProvider for FolderModProvider {
	fn get_id() -> ModProviderId {
		ModProviderId::Folder
	}

	fn default() -> Result<Self> {
		Ok(Self {
			folder_path: app_paths::local_mods_path()?,
		})
	}

	async fn insert_mods(&self, db: &DbMutex) -> Result {
		let source_hash = self.compute_source_hash();

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

			db.insert_mod(&game_mod, Self::get_id(), &source_hash);
		}

		Ok(())
	}
}
