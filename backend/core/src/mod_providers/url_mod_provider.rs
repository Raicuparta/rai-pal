use std::{
	fs,
	hash::{DefaultHasher, Hash, Hasher},
	path::PathBuf,
};

use rai_pal_proc_macros::serializable_struct;

use super::mod_provider::ModProvider;
use crate::{
	app_paths, http,
	local_database::{
		app_database::DbMutex,
		mod_database::{ModDatabase, compute_scope, scope_id},
	},
	mod_providers::mod_provider::ModProviderId,
	mods::game_mod::GameMod,
	path_extensions::PathExt,
	result::{LogErrExt, Result},
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
const DATABASE_VERSION: i32 = 3;

fn default_url() -> String {
	format!("{URL_BASE}/{DATABASE_VERSION}/mods.json")
}

fn compute_source_hash(url: &str) -> String {
	let mut hasher = DefaultHasher::new();
	url.hash(&mut hasher);
	hasher.finish().to_string()
}

#[derive(Default)]
#[serializable_struct]
pub struct UrlModSource {
	pub url: String,
	pub enabled: bool,
}

#[serializable_struct]
pub struct UrlModSources {
	#[serde(default = "default_true")]
	pub default_enabled: bool,
	#[serde(default)]
	pub sources: Vec<UrlModSource>,
}

impl Default for UrlModSources {
	fn default() -> Self {
		Self {
			default_enabled: true,
			sources: Vec::new(),
		}
	}
}

fn default_true() -> bool {
	true
}

// If the mod sources schema changes, update this so it gets recreated.
const URL_MOD_SOURCES_VERSION: u32 = 1u32;

fn url_mod_sources_path() -> Result<PathBuf> {
	app_paths::app_data_file(&format!("url-mod-sources-{URL_MOD_SOURCES_VERSION}.json"))
}

fn read_url_mod_sources() -> UrlModSources {
	let default = UrlModSources::default();

	let Ok(path) = url_mod_sources_path() else {
		return default;
	};

	if !path.is_file() {
		return default;
	}

	let mut sources = fs::read_to_string(&path)
		.ok_or_log("Failed to read URL mod sources")
		.and_then(|data| serde_json::from_str(&data).ok_or_log("Failed to parse URL mod sources"))
		.unwrap_or(default);

	let default_url = default_url();
	sources.sources.retain(|source| source.url != default_url);

	sources
}

fn write_url_mod_sources(sources: &UrlModSources) -> Result {
	let path = url_mod_sources_path()?;

	fs::create_dir_all(path.try_parent()?)?;
	fs::write(&path, serde_json::to_string(sources)?)?;

	Ok(())
}

#[serializable_struct]
pub struct UrlModSourcesResponse {
	pub default_source: UrlModSource,
	pub sources: Vec<UrlModSource>,
}

pub async fn get_mods_from_url_mod_source(url: &str) -> Result<Vec<GameMod>> {
	let mods = http::CLIENT
		.get(url)
		.timeout(std::time::Duration::from_secs(15))
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

	if url != default_url() && !sources.sources.iter().any(|source| source.url == url) {
		sources.sources.push(UrlModSource { url, enabled: true });
		write_url_mod_sources(&sources)?;
	}

	Ok(())
}

pub fn remove_url_mod_source(url: &str) -> Result {
	let mut sources = read_url_mod_sources();

	sources.sources.retain(|source| source.url != url);
	write_url_mod_sources(&sources)
}

pub fn set_url_mod_source_enabled(url: &str, enabled: bool) -> Result {
	let mut sources = read_url_mod_sources();

	if url == default_url() {
		sources.default_enabled = enabled;
	} else if let Some(source) = sources.sources.iter_mut().find(|source| source.url == url) {
		source.enabled = enabled;
	}

	write_url_mod_sources(&sources)
}

pub fn get_url_mod_sources() -> UrlModSourcesResponse {
	let sources = read_url_mod_sources();

	UrlModSourcesResponse {
		default_source: UrlModSource {
			url: default_url(),
			enabled: sources.default_enabled,
		},
		sources: sources.sources,
	}
}

pub struct UrlModProvider;

impl ModProvider for UrlModProvider {
	fn get_id() -> ModProviderId {
		ModProviderId::Url
	}

	fn default() -> Result<Self> {
		Ok(Self)
	}

	async fn refresh(&self, db: &DbMutex) -> Result {
		let sources = read_url_mod_sources();

		let mut enabled_sources = Vec::new();

		if sources.default_enabled {
			enabled_sources.push((default_url(), String::new()));
		}

		enabled_sources.extend(
			sources
				.sources
				.iter()
				.filter(|source| source.enabled)
				.map(|source| (source.url.clone(), compute_source_hash(&source.url))),
		);

		let mut keep_ids = Vec::new();

		for (url, source_hash) in enabled_sources {
			let scope = compute_scope(Self::get_id(), &source_hash);

			match fetch_and_insert(&url, &source_hash, &scope, db).await {
				Ok(ids) => keep_ids.extend(ids),
				Err(error) => {
					// If a source fails to refresh, keep whatever we already had
					// for it as a fallback, but still remove disabled sources.
					log::warn!(
						"Failed to refresh mod source `{url}`, keeping existing mods as fallback: {error}"
					);
					keep_ids.extend(db.get_mod_ids_in_scope(Self::get_id(), &scope)?);
				}
			}
		}

		db.remove_mods_except(Self::get_id(), &keep_ids)?;

		Ok(())
	}
}

async fn fetch_and_insert(
	url: &str,
	source_hash: &str,
	scope: &str,
	db: &DbMutex,
) -> Result<Vec<String>> {
	// The timeout bounds how long a refresh can hold the mod refresh lock,
	// so a single unresponsive mod source can't block all future refreshes.
	let mods = http::CLIENT
		.get(url)
		.timeout(std::time::Duration::from_secs(15))
		.send()
		.await?
		.json::<UrlModDatabase>()
		.await?
		.mods;

	let mut ids = Vec::with_capacity(mods.len());

	for game_mod in &mods {
		db.insert_mod(game_mod, ModProviderId::Url, source_hash);
		ids.push(scope_id(scope, &game_mod.id).into_owned());
	}

	Ok(ids)
}
