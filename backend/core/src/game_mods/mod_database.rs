use std::{
	collections::HashMap,
	fs::{
		self,
		File,
	},
	path::Path,
};

use rai_pal_proc_macros::{
	serializable_enum,
	serializable_struct,
};
use zip::ZipArchive;

use crate::{
	architecture::Architecture,
	files,
	game_engines::{
		game_engine::{
			EngineBrand,
			EngineVersionRange,
		},
		unity::UnityBackend,
	},
	game_mods::mod_config::ModConfig,
	http,
	local_mod,
	operating_system::OperatingSystem,
	paths,
	result::Result,
};

const URL_BASE: &str = "https://raicuparta.github.io/rai-pal-db/mod-db";

// The repository over at github.com/Raicuparta/rai-pal-db can have multiple versions of the database.
// This way we prevent old versions of Rai Pal from breaking unless we want them to.
// So when you need to change the database in a backwards-incompatible way,
// you would create a new folder in the database repository and change this number to match the folder.
const DATABASE_VERSION: i32 = 1;

#[serializable_struct]
pub struct GameMod {
	pub id: String,
	pub title: String,
	pub author: String,
	pub source_code: String,
	pub description: String,
	pub latest_version: ModDownload,
	pub engine: Option<EngineBrand>,
	pub engine_version_range: Option<EngineVersionRange>,
	pub unity_backend: Option<UnityBackend>,
	pub architecture: Option<Architecture>,
	pub game_os: Option<OperatingSystem>,
	pub host_os: Option<OperatingSystem>,
	pub deprecated: Option<bool>,
	pub config: Option<ModConfig>,
	pub dependencies: Option<Vec<ModDependency>>,
	pub install: Option<ModInstall>,
	pub run_for_game: Option<ModRunForGame>,
}

#[serializable_struct]
pub struct ModInstallExtract {
	pub source: String,
	pub destination: String,
}

#[serializable_struct]
pub struct ModInstallWrite {
	pub content: String,
	pub destination: String,
}

#[serializable_struct]
pub struct ModInstall {
	pub extract: Option<Vec<ModInstallExtract>>,
	pub write: Option<Vec<ModInstallWrite>>,
	pub wine_dll_overrides: Option<Vec<String>>,
	pub main_installed_folder_path: Option<String>,
}

#[serializable_struct]
pub struct ModRunForGame {
	pub path: Option<String>,
	pub args: Option<Vec<String>>,
	pub wine_environment: Option<HashMap<String, String>>,
}

#[serializable_struct]
pub struct ModDatabase {
	pub mods: Vec<GameMod>,
}

#[serializable_struct]
pub struct ModDownload {
	pub id: String,
	pub url: String,
}

#[serializable_struct]
pub struct ModDependency {
	pub mod_id: String,
}

#[serializable_enum]
pub enum ModConfigDestinationType {
	File,
	Folder,
}

pub async fn get() -> Result<ModDatabase> {
	Ok(http::CLIENT
		.get(format!("{URL_BASE}/{DATABASE_VERSION}/mods.json"))
		.send()
		.await?
		.json::<ModDatabase>()
		.await?)
}

impl GameMod {
	pub const FILE_NAME: &'static str = "rai-pal-manifest.json";

	pub fn from_file(path: &Path) -> Option<Self> {
		match fs::read_to_string(path)
			.and_then(|manifest_bytes| Ok(serde_json::from_str::<Self>(&manifest_bytes)?))
		{
			Ok(manifest) => Some(manifest),
			Err(error) => {
				log::error!(
					"Error getting manifest in path '{}': {}",
					path.display(),
					error
				);
				None
			}
		}
	}

	pub async fn download(&self) -> Result {
		let target_path = paths::local_mods_path()?.join(&self.id);
		let downloads_path = paths::downloads_path()?;
		let mod_id = &self.id;

		let response = reqwest::get(&self.latest_version.url).await?;

		fs::create_dir_all(&downloads_path)?;

		let zip_path = downloads_path.join(format!("{mod_id}.zip"));

		// TODO Stream to disk instead of keeping it all in memory.
		fs::write(&zip_path, response.bytes().await?)?;
		let file = File::open(&zip_path)?;

		let mut zip_archive = ZipArchive::new(file)?;

		files::extract(&mut zip_archive, &target_path)?;

		fs::write(
			local_mod::get_manifest_path(&target_path),
			serde_json::to_string_pretty(&self)?,
		)?;

		Ok(())
	}
}
