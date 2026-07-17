use std::{
	fs,
	path::PathBuf,
};

use rai_pal_proc_macros::serializable_struct;

use super::mod_provider::ModProvider;
use crate::{
	app_paths,
	http,
	local_database::{
		app_database::DbMutex,
		mod_database::ModDatabase,
	},
	mod_providers::mod_provider::ModProviderId,
	mods::game_mod::GameMod,
	path_extensions::PathExt,
	result::{
		LogErrExt,
		Result,
	},
};

#[serializable_struct]
pub struct UrlModDatabase {
	pub mods: Vec<GameMod>,
}

const URL_BASE: &str = "https://raicuparta.github.io/rai-pal-db/mod-db";

// The repository over at github.com/Raicuparta/rai-pal-db can have multiple versions of the database.
// This way we prevent old versions of Rai Pal from breaking unless we want them to.
// So when you need to change the database in a backwards-incompatible way,
// you would create a new folder in the database repository and change this number to match the folder.
const DATABASE_VERSION: i32 = 1;

fn default_url() -> String {
	format!("{URL_BASE}/{DATABASE_VERSION}/mods.json")
}

#[derive(Default)]
#[serializable_struct]
pub struct UrlModSources {
	pub additional_urls: Vec<String>,
}

fn url_mod_sources_path() -> Result<PathBuf> {
	app_paths::app_data_file("url-mod-sources.json")
}

fn read_url_mod_sources() -> UrlModSources {
	let Ok(path) = url_mod_sources_path() else {
		return UrlModSources::default();
	};

	if !path.is_file() {
		return UrlModSources::default();
	}

	fs::read_to_string(&path)
		.ok_or_log("Failed to read URL mod sources")
		.and_then(|data| serde_json::from_str(&data).ok_or_log("Failed to parse URL mod sources"))
		.unwrap_or_default()
}

fn write_url_mod_sources(sources: &UrlModSources) -> Result {
	let path = url_mod_sources_path()?;

	fs::create_dir_all(path.try_parent()?)?;
	fs::write(&path, serde_json::to_string(sources)?)?;

	Ok(())
}

pub async fn get_mods_from_url_mod_source(url: &str) -> Result<Vec<GameMod>> {
	let mods = http::CLIENT
		.get(url)
		.send()
		.await?
		.error_for_status()?
		.json::<UrlModDatabase>()
		.await?
		.mods;

	Ok(mods)
}

pub fn add_url_mod_source(url: String) -> Result {
	let mut sources = read_url_mod_sources();

	if !sources.additional_urls.contains(&url) {
		sources.additional_urls.push(url);
		write_url_mod_sources(&sources)?;
	}

	Ok(())
}

pub fn remove_url_mod_source(url: &str) -> Result {
	let mut sources = read_url_mod_sources();

	sources.additional_urls.retain(|u| u != url);
	write_url_mod_sources(&sources)
}

pub fn get_url_mod_sources() -> UrlModSources {
	read_url_mod_sources()
}

pub fn get_all_urls() -> Vec<String> {
	let mut urls = vec![default_url()];

	urls.extend(read_url_mod_sources().additional_urls);

	urls
}

pub struct UrlModProvider {
	pub urls: Vec<String>,
}

impl ModProvider for UrlModProvider {
	fn get_id() -> ModProviderId {
		ModProviderId::Url
	}

	fn default() -> Result<Self> {
		Ok(Self {
			urls: get_all_urls(),
		})
	}

	async fn insert_mods(&self, db: &DbMutex) -> Result {
		for url in &self.urls {
			http::CLIENT
				.get(url)
				.send()
				.await?
				.json::<UrlModDatabase>()
				.await?
				.mods
				.iter()
				.for_each(|game_mod| db.insert_mod(game_mod, Self::get_id()));
		}

		Ok(())
	}
}
