use std::{
	collections::HashMap,
	fs::{
		self,
	},
	path::{
		Path,
		PathBuf,
	},
};

use enum_dispatch::enum_dispatch;
use log::error;
use rai_pal_proc_macros::{
	serializable_enum,
	serializable_struct,
};
use serde::Deserialize;

use super::{
	bepinex::BepInEx,
	runnable_loader::RunnableLoader,
	ue4ss::Ue4ss,
};
use crate::{
	game::DbGame,
	game_engines::game_engine::EngineBrand,
	local_mod::{
		LocalMod,
		ModKind,
	},
	mod_loaders::{
		mod_database::{
			ModConfigDestinationType,
			ModConfigs,
		},
		package::PackageLoader,
	},
	paths::{
		self,
		open_folder_or_parent,
	},
	remote_config,
	result::Result,
};

#[serializable_struct]
pub struct ModLoaderData {
	pub id: ModLoaderId,
	pub path: PathBuf,
	pub kind: ModKind,
	pub engine: Option<EngineBrand>,
}

#[derive(Deserialize)]
pub struct LoaderBuild {
	pub os: String,
	#[serde(rename = "downloadUrl")]
	pub download_url: String,
	#[serde(rename = "unityBackend")]
	pub unity_backend: Option<String>,
	pub arch: Option<String>,
}

#[derive(Deserialize)]
pub struct LoaderRelease {
	pub version: String,
	pub timestamp: u64,
	pub builds: Vec<LoaderBuild>,
}

#[derive(Deserialize)]
pub struct LoaderDatabase {
	pub id: String,
	pub releases: Vec<LoaderRelease>,
}

#[serializable_enum]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase", ascii_case_insensitive)]
pub enum ModLoaderId {
	BepInEx,
	Ue4ss,
	Runnable,
	Package,
}

impl ModLoaderId {
	pub const fn as_str(self) -> &'static str {
		match self {
			Self::BepInEx => "bepinex",
			Self::Ue4ss => "ue4ss",
			Self::Runnable => "runnable",
			Self::Package => "package",
		}
	}
}

#[serializable_struct]
pub struct ModLoaderStatus {
	pub installed_version: Option<String>,
	pub latest_version: Option<String>,
}

#[serializable_struct]
pub struct LocalModLoaderData {
	pub installed_version: Option<String>,
}

#[serializable_struct]
pub struct RemoteModLoaderData {
	pub latest_version: Option<String>,
}

#[serializable_struct]
pub struct LocalModLoader {
	pub common: ModLoaderData,
	pub data: LocalModLoaderData,
}

#[serializable_struct]
pub struct RemoteModLoader {
	pub common: ModLoaderData,
	pub data: RemoteModLoaderData,
}

pub type LocalModLoadersMap = HashMap<String, LocalModLoader>;
pub type RemoteModLoadersMap = HashMap<String, RemoteModLoader>;

#[enum_dispatch]
#[derive(Clone)]
pub enum ModLoader {
	BepInEx,
	Ue4ss,
	RunnableLoader,
	PackageLoader,
}

#[enum_dispatch(ModLoader)]
pub trait ModLoaderActions {
	async fn install_loader(&self, game: &DbGame, local_mod: &LocalMod) -> Result {
		self.install_mod_inner(game, local_mod).await
	}
	async fn uninstall_loader(&self, game: &DbGame, local_mod: &LocalMod) -> Result {
		self.uninstall_mod_inner(game, local_mod).await
	}
	async fn install_mod_inner(&self, game: &DbGame, local_mod: &LocalMod) -> Result;
	async fn uninstall_mod_inner(&self, game: &DbGame, local_mod: &LocalMod) -> Result;
	async fn run_without_game(&self, local_mod: &LocalMod) -> Result;
	fn get_config_path(&self, game: &DbGame, mod_configs: &ModConfigs) -> Result<PathBuf>;
	fn open_installed_mod_folder(&self, game: &DbGame, local_mod: &LocalMod) -> Result;
	fn get_data(&self) -> &ModLoaderData;
	fn get_wine_dll_overrides(&self, _game: &DbGame) -> Vec<String> {
		Vec::new()
	}

	fn open_folder(&self) -> Result {
		open_folder_or_parent(&self.get_data().path)
	}

	async fn install_mod(&self, game: &DbGame, local_mod: &LocalMod) -> Result {
		if local_mod.common.is_loader.unwrap_or_default() {
			self.install_loader(game, local_mod).await?;
		} else {
			self.install_mod_inner(game, local_mod).await?;
		}

		self.update_installed_mod_manifest(local_mod, game)?;

		Ok(())
	}

	async fn uninstall_mod(&self, game: &DbGame, local_mod: &LocalMod) -> Result {
		log::info!(
			"Uninstalling mod '{}' for game '{}'",
			local_mod.common.id,
			game.display_title
		);
		if local_mod.common.is_loader.unwrap_or_default() {
			self.uninstall_loader(game, local_mod).await?;
		} else {
			self.uninstall_mod_inner(game, local_mod).await?;
		}

		let manifest_path = game.get_installed_mod_manifest_path(&local_mod.common.id)?;
		if manifest_path.is_file() {
			fs::remove_file(manifest_path)?;
		}

		Ok(())
	}

	fn update_installed_mod_manifest(&self, local_mod: &LocalMod, game: &DbGame) -> Result {
		if self.get_data().kind != ModKind::Runnable {
			let manifest_path = game.get_installed_mod_manifest_path(&local_mod.common.id)?;
			fs::create_dir_all(paths::path_parent(&manifest_path)?)?;
			let manifest_contents = serde_json::to_string_pretty(&local_mod.data.manifest)?;
			fs::write(manifest_path, manifest_contents)?;
		}

		Ok(())
	}

	async fn download_config(
		&self,
		game: &DbGame,
		mod_configs: &ModConfigs,
		config_file: &str,
		overwrite: bool,
	) -> Result {
		let destination_path = self.get_config_path(game, mod_configs)?;

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

		match mod_configs.destination_type {
			ModConfigDestinationType::File => {
				remote_config::download_config_file(config_file, game, &destination_path).await?;
			}
			ModConfigDestinationType::Folder => {
				remote_config::download_config_folder(config_file, game, &destination_path).await?;
			}
		}

		Ok(())
	}

	fn configure_mod(&self, game: &DbGame, local_mod: &LocalMod, open_folder: bool) -> Result {
		if let Some(configs) = local_mod.data.manifest.configs.as_ref() {
			let config_path = self.get_config_path(game, configs)?;
			if open_folder {
				paths::open_folder_or_parent(&config_path)?;
			} else {
				open::that_detached(config_path)?;
			}
		}

		Ok(())
	}
}

pub trait ModLoaderStatic {
	const ID: ModLoaderId;

	fn new(resources_path: &Path) -> Result<Self>
	where
		Self: Sized;
}

pub type Map = HashMap<ModLoaderId, ModLoader>;
pub type DataMap = HashMap<ModLoaderId, ModLoaderData>;

fn create_map_entry<TModLoader: ModLoaderActions + ModLoaderStatic>(
	path: &Path,
) -> Result<(ModLoaderId, ModLoader)>
where
	ModLoader: std::convert::From<TModLoader>,
{
	let mod_loader: ModLoader = TModLoader::new(path)?.into();

	Ok((TModLoader::ID, mod_loader))
}

fn add_entry<TModLoader: ModLoaderActions + ModLoaderStatic>(path: &Path, map: &mut Map)
where
	ModLoader: std::convert::From<TModLoader>,
{
	match create_map_entry::<TModLoader>(path) {
		Ok((key, value)) => {
			map.insert(key, value);
		}
		Err(err) => error!("Failed to create map entry: {err}"),
	}
}

pub fn get_map(resources_path: &Path) -> Map {
	let mut map = Map::new();

	add_entry::<BepInEx>(resources_path, &mut map);
	add_entry::<Ue4ss>(resources_path, &mut map);
	add_entry::<RunnableLoader>(resources_path, &mut map);
	add_entry::<PackageLoader>(resources_path, &mut map);

	map
}

pub fn get_local_mod_loaders_map(map: &Map) -> Result<LocalModLoadersMap> {
	map.values()
		.map(|mod_loader| {
			let common = mod_loader.get_data().clone();
			Ok((
				common.id.as_str().to_string(),
				LocalModLoader {
					common,
					data: LocalModLoaderData {
						installed_version: None,
					},
				},
			))
		})
		.collect()
}

pub fn get_remote_mod_loaders_map(map: &Map) -> Result<RemoteModLoadersMap> {
	map.values()
		.map(|mod_loader| {
			let common = mod_loader.get_data().clone();
			Ok((
				common.id.as_str().to_string(),
				RemoteModLoader {
					common,
					data: RemoteModLoaderData {
						latest_version: None,
					},
				},
			))
		})
		.collect()
}

fn config_exists(path: &Path) -> Result<bool> {
	if !path.try_exists()? {
		return Ok(false);
	}

	if path.is_dir() && fs::read_dir(path)?.next().is_none() {
		return Ok(false);
	}

	Ok(true)
}
