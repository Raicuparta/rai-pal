use std::{
	collections::HashMap,
	path::PathBuf,
};

use rai_pal_proc_macros::{
	serializable_enum,
	serializable_struct,
};

use crate::{
	architecture::Architecture,
	game_engines::{
		game_engine::EngineBrand,
		unity::UnityBackend,
	},
	game_mods::{
		game_mod::EngineVersionRange,
		mod_config::ModConfig,
	},
	http,
	operating_system::OperatingSystem,
	result::Result,
};

const URL_BASE: &str = "https://raicuparta.github.io/rai-pal-db/mod-db";

// The repository over at github.com/Raicuparta/rai-pal-db can have multiple versions of the database.
// This way we prevent old versions of Rai Pal from breaking unless we want them to.
// So when you need to change the database in a backwards-incompatible way,
// you would create a new folder in the database repository and change this number to match the folder.
const DATABASE_VERSION: i32 = 1;

#[serializable_struct]
pub struct DatabaseEntry {
	pub id: String,
	pub title: String,
	pub author: String,
	pub source_code: String,
	pub description: String,
	pub latest_version: Option<ModDownload>,
	pub engine: Option<EngineBrand>,
	pub engine_version_range: Option<EngineVersionRange>,
	pub unity_backend: Option<UnityBackend>,
	pub architecture: Option<Architecture>,
	pub game_os: Option<OperatingSystem>,
	pub host_os: Option<OperatingSystem>,
	pub redownload_id: Option<i32>,
	pub deprecated: Option<bool>,
	pub config: Option<ModConfig>,
	pub dependencies: Option<Vec<ModDependency>>,
}

#[serializable_struct]
pub struct RunnableModData {
	pub path: String,
	pub args: Vec<String>,
	pub wine_environment: Option<HashMap<String, String>>,
}

#[serializable_struct]
pub struct ModDatabase {
	pub mods: Vec<DatabaseEntry>,
}

#[serializable_struct]
pub struct ModDownload {
	pub id: String,
	pub url: String,
	pub root: Option<PathBuf>,
	pub runnable: Option<RunnableModData>,
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

#[serializable_struct]
pub struct ModGithubInfo {
	pub user: String,
	pub repo: String,
	pub asset_name: String,
	pub root: Option<PathBuf>,
	pub runnable: Option<RunnableModData>,
}

pub async fn get() -> Result<ModDatabase> {
	Ok(http::CLIENT
		.get(format!("{URL_BASE}/{DATABASE_VERSION}/mods.json"))
		.send()
		.await?
		.json::<ModDatabase>()
		.await?)
}

impl DatabaseEntry {
	pub fn get_download(&self) -> Option<ModDownload> {
		self.latest_version.clone().map(|mut download| {
			if let Some(redownload_id) = self.redownload_id {
				download.id = format!("{}/{}", download.id, redownload_id);
			}

			download
		})
	}
}
