use std::{
	fs,
	hash::{
		DefaultHasher,
		Hash,
		Hasher,
	},
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

fn compute_source_hash(url: &str, is_default: bool) -> String {
	if is_default {
		return String::new();
	}
	let mut hasher = DefaultHasher::new();
	url.hash(&mut hasher);
	hasher.finish().to_string()
}

#[derive(Default)]
#[serializable_struct]
pub struct UrlModSource {
	pub url: String,
	pub is_default: bool,
	pub enabled: bool,
}

#[derive(Default)]
#[serializable_struct]
pub struct UrlModSources {
	pub sources: Vec<UrlModSource>,
}

fn default_source() -> UrlModSource {
	UrlModSource {
		url: default_url(),
		is_default: true,
		enabled: true,
	}
}

fn default_sources() -> UrlModSources {
	UrlModSources {
		sources: vec![default_source()],
	}
}

fn url_mod_sources_path() -> Result<PathBuf> {
	app_paths::app_data_file("url-mod-sources.json")
}

fn read_url_mod_sources() -> UrlModSources {
	let default = default_sources();

	let Ok(path) = url_mod_sources_path() else {
		return default;
	};

	if !path.is_file() {
		return default;
	}

	fs::read_to_string(&path)
		.ok_or_log("Failed to read URL mod sources")
		.and_then(|data| serde_json::from_str(&data).ok_or_log("Failed to parse URL mod sources"))
		.unwrap_or(default)
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

	if !sources.sources.iter().any(|source| source.url == url) {
		sources.sources.push(UrlModSource {
			url,
			is_default: false,
			enabled: true,
		});
		write_url_mod_sources(&sources)?;
	}

	Ok(())
}

pub fn remove_url_mod_source(url: &str) -> Result {
	let mut sources = read_url_mod_sources();

	sources.sources.retain(|source| source.url != url || source.is_default);
	write_url_mod_sources(&sources)
}

pub fn set_url_mod_source_enabled(url: &str, enabled: bool) -> Result {
	let mut sources = read_url_mod_sources();

	if let Some(source) = sources.sources.iter_mut().find(|source| source.url == url) {
		source.enabled = enabled;
		write_url_mod_sources(&sources)?;
	}

	Ok(())
}

pub fn get_url_mod_sources() -> UrlModSources {
	read_url_mod_sources()
}

pub fn get_enabled_sources() -> Vec<UrlModSource> {
	read_url_mod_sources()
		.sources
		.into_iter()
		.filter(|source| source.enabled)
		.collect()
}

pub struct UrlModProvider {
	pub sources: Vec<UrlModSource>,
}

impl ModProvider for UrlModProvider {
	fn get_id() -> ModProviderId {
		ModProviderId::Url
	}

	fn default() -> Result<Self> {
		Ok(Self {
			sources: get_enabled_sources(),
		})
	}

	async fn insert_mods(&self, db: &DbMutex) -> Result {
		for source in &self.sources {
			let source_hash = compute_source_hash(&source.url, source.is_default);

			http::CLIENT
				.get(&source.url)
				.send()
				.await?
				.json::<UrlModDatabase>()
				.await?
				.mods
				.iter()
				.for_each(|game_mod| db.insert_mod(game_mod, Self::get_id(), &source_hash));
		}

		Ok(())
	}
}
