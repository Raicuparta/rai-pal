use fs_err as fs;
use std::collections::BTreeMap;

use crate::{
	app_paths, database_url, http,
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

fn url_mod_sources_path() -> Result<std::path::PathBuf> {
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

pub async fn fetch_mod_database(url: &str) -> Result<UrlModDatabase> {
	let database = http::CLIENT
		.get(url)
		.timeout(std::time::Duration::from_secs(15))
		.send()
		.await?
		.error_for_status()?
		.json::<UrlModDatabase>()
		.await?;

	Ok(database)
}

pub async fn get_mods_from_url_mod_source(url: &str) -> Result<Vec<Mod>> {
	Ok(fetch_mod_database(url).await?.mods)
}

pub fn enabled_sources() -> Vec<String> {
	let sources = read_url_mod_sources();
	let mut urls = Vec::new();

	if sources.default_enabled {
		urls.push(default_url());
	}

	urls.extend(
		sources
			.sources
			.iter()
			.filter(|source| source.enabled)
			.map(|source| source.url.clone()),
	);

	urls
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

pub fn move_url_mod_source(url: &str, up: bool) -> Result {
	let mut sources = read_url_mod_sources();

	let Some(index) = sources.sources.iter().position(|source| source.url == url) else {
		return Ok(());
	};

	let target = if up {
		index.checked_sub(1)
	} else {
		Some(index + 1).filter(|target| *target < sources.sources.len())
	};

	if let Some(target) = target {
		sources.sources.swap(index, target);
		write_url_mod_sources(&sources)?;
	}

	Ok(())
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
