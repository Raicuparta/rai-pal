use fs_err as fs;
use std::{
	collections::BTreeMap,
	hash::{DefaultHasher, Hash, Hasher},
	path::PathBuf,
};

use super::mod_provider::ModProvider;
use crate::{
	app_paths, database_url, http,
	local_database::{
		app_database::Db,
		mod_database::{ModDatabase, compute_scope, scope_id},
	},
	mod_providers::mod_provider::ModProviderId,
	mods::{manifest::Mod, mod_tag::ModTag},
	path_extensions::PathExt,
	result::{LogErrExt, Result},
};

#[derive(serde::Deserialize)]
pub struct UrlModDatabase {
	#[serde(default)]
	pub tags: BTreeMap<String, ModTag>,
	pub mods: Vec<Mod>,
}

// The database can serve multiple versions.
// This way we prevent old versions of Rai Pal from breaking unless we want them to.
// So when you need to change the database in a backwards-incompatible way,
// you would create a new folder in the database repository and change this number to match the folder.
const DATABASE_VERSION: i32 = 4;

fn default_url() -> String {
	format!(
		"{}/mod-db/{DATABASE_VERSION}/mods.json",
		database_url::base_url()
	)
}

fn compute_source_hash(url: &str) -> String {
	let mut hasher = DefaultHasher::new();
	url.hash(&mut hasher);
	hasher.finish().to_string()
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct UrlModSource {
	pub url: String,
	pub enabled: bool,
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
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

const fn default_true() -> bool {
	true
}

// If the mod sources schema changes, update this so it gets recreated.
const URL_MOD_SOURCES_VERSION: u32 = 1u32;

fn url_mod_sources_path() -> Result<PathBuf> {
	app_paths::app_data_file(&format!("url-mod-sources-{URL_MOD_SOURCES_VERSION}.json"))
}

fn read_url_mod_sources() -> UrlModSources {
	let default = UrlModSources::default();

	let Some(path) = url_mod_sources_path().ok_or_log("Failed to get URL mod sources path") else {
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

pub struct UrlModSourcesResponse {
	pub default_source: UrlModSource,
	pub sources: Vec<UrlModSource>,
}

pub async fn get_mods_from_url_mod_source(url: &str) -> Result<Vec<Mod>> {
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

	async fn refresh(&self, db: &Db) -> Result {
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
		let mut keep_tag_ids = Vec::new();

		for (url, source_hash) in enabled_sources {
			let scope = compute_scope(Self::get_id(), &source_hash);

			match fetch_and_insert(&url, &source_hash, &scope, db).await {
				Ok((ids, tag_ids)) => {
					keep_ids.extend(ids);
					keep_tag_ids.extend(tag_ids);
				}
				Err(error) => {
					// If a source fails to refresh, keep whatever we already had
					// for it as a fallback, but still remove disabled sources.
					log::warn!(
						"Failed to refresh mod source `{url}`, keeping existing mods as fallback: {error}"
					);
					keep_ids.extend(db.get_mod_ids_in_scope(Self::get_id(), &scope)?);
					keep_tag_ids.extend(db.get_tag_ids_in_scope(Self::get_id(), &scope)?);
				}
			}
		}

		db.remove_mods_except(Self::get_id(), &keep_ids)?;
		db.remove_mod_tags_except(Self::get_id(), &keep_tag_ids)?;

		Ok(())
	}
}

async fn fetch_and_insert(
	url: &str,
	source_hash: &str,
	scope: &str,
	db: &Db,
) -> Result<(Vec<String>, Vec<String>)> {
	// The timeout bounds how long a refresh can hold the mod refresh lock,
	// so a single unresponsive mod source can't block all future refreshes.
	let database = http::CLIENT
		.get(url)
		.timeout(std::time::Duration::from_secs(15))
		.send()
		.await?
		.json::<UrlModDatabase>()
		.await?;

	db.insert_mod_tags(&database.tags, ModProviderId::Url, source_hash);

	db.insert_mods(&database.mods, ModProviderId::Url, source_hash);

	let ids = database
		.mods
		.iter()
		.map(|game_mod| scope_id(scope, &game_mod.id).into_owned())
		.collect::<Vec<_>>();

	let tag_ids = database
		.tags
		.keys()
		.map(|tag_id| scope_id(scope, tag_id).into_owned())
		.collect();

	Ok((ids, tag_ids))
}
