use std::{
	collections::HashMap,
	fs,
	path::{
		Path,
		PathBuf,
	},
};

use rai_pal_proc_macros::{
	serializable_enum,
	serializable_struct,
};

use crate::{
	game::DbGame,
	game_mods::{
		game_mod::CommonModData,
		mod_config::ModConfig,
	},
	mod_manifest::{
		self,
		Manifest,
	},
	paths::{
		self,
		open_folder_or_parent,
	},
	result::{
		Error,
		LogErrExt,
		Result,
	},
};

#[serializable_enum]
pub enum ModKind {
	Installable,
	Runnable,
}

#[serializable_struct]
pub struct LocalModData {
	pub path: PathBuf,
	pub manifest: Manifest,
}

#[serializable_struct]
pub struct LocalMod {
	pub data: LocalModData,
	pub common: CommonModData,
}

pub fn get_manifest_path(mod_path: &Path) -> PathBuf {
	mod_path.join(mod_manifest::Manifest::FILE_NAME)
}

impl LocalMod {
	pub fn new(manifest_path: &Path) -> Result<Self> {
		let manifest = mod_manifest::get(manifest_path)
			.ok_or_else(|| Error::ManifestNotFound(manifest_path.display().to_string()))?;

		let mod_path = paths::path_parent(manifest_path)?;

		Ok(Self {
			data: LocalModData {
				path: mod_path.to_path_buf(),
				manifest: manifest.clone(),
			},
			common: CommonModData {
				id: paths::file_name_without_extension(mod_path)?.to_string(),
				engine: manifest.engine,
				engine_version_range: manifest.engine_version_range,
				architecture: manifest.architecture,
				unity_backend: manifest.unity_backend,
			},
		})
	}

	pub fn open_folder(&self) -> Result {
		open_folder_or_parent(&self.data.path)
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
		let manifest_path = game.get_installed_mod_manifest_path(&self.common.id)?;
		fs::create_dir_all(paths::path_parent(&manifest_path)?)?;
		let manifest_contents = serde_json::to_string_pretty(&self.data.manifest)?;
		fs::write(manifest_path, manifest_contents)?;

		Ok(())
	}

	pub fn get_config_path(&self, config: &ModConfig, _game: &DbGame) -> Result<PathBuf> {
		// TODO: handle tokens and game.
		Ok(PathBuf::from(&config.destination_path))
	}

	pub fn configure_mod(&self, game: &DbGame, open_folder: bool) -> Result {
		if let Some(config) = self.data.manifest.config.as_ref() {
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
}

pub fn get_all() -> Result<HashMap<String, LocalMod>> {
	Ok(paths::glob_path(
		&paths::local_mods_path()?
			.join("*")
			.join(mod_manifest::Manifest::FILE_NAME),
	)
	.iter()
	.filter_map(|manifest_path| {
		LocalMod::new(manifest_path)
			.ok_or_log("Failed to create local mod")
			.map(|local_mod| (local_mod.common.id.clone(), local_mod))
	})
	.collect())
}

pub type Map = HashMap<String, LocalMod>;

pub fn delete(local_mod: &LocalMod) -> Result {
	if local_mod.data.path.exists() {
		fs::remove_dir_all(&local_mod.data.path)?;
	}

	Ok(())
}
