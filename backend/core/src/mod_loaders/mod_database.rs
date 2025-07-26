use std::path::PathBuf;

use lazy_regex::regex_captures;
use log::error;
use rai_pal_proc_macros::{serializable_enum, serializable_struct};

use crate::{
	game_engines::{game_engine::EngineBrand, unity::UnityBackend},
	game_mod::EngineVersionRange,
	http_client,
	result::Result,
};

const URL_BASE: &str = "https://raicuparta.github.io/rai-pal-db/mod-db";

// The repository over at github.com/Raicuparta/rai-pal-db can have multiple versions of the database.
// This way we prevent old versions of Rai Pal from breaking unless we want them to.
// So when you need to change the database in a backwards-incompatible way,
// you would create a new folder in the database repository and change this number to match the folder.
const DATABASE_VERSION: i32 = 0;

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
	pub github: Option<ModGithubInfo>,
	pub redownload_id: Option<i32>,
	pub deprecated: Option<bool>,
	pub configs: Option<ModConfigs>,
}

#[serializable_struct]
pub struct RunnableModData {
	pub path: String,
	pub args: Vec<String>,
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
pub struct ModConfigs {
	pub destination_path: String,
	pub destination_type: ModConfigDestinationType,
	pub mod_id_override: Option<String>,
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

pub async fn get(mod_loader_id: &str) -> Result<ModDatabase> {
	let client = http_client::get_client();
	Ok(client
		.get(format!(
			"{URL_BASE}/{DATABASE_VERSION}/{mod_loader_id}.json"
		))
		.send()
		.await?
		.json::<ModDatabase>()
		.await?)
}

impl DatabaseEntry {
	pub async fn get_download(&self) -> Option<ModDownload> {
		self.get_download_inner().await.map(|mut download| {
			if let Some(redownload_id) = self.redownload_id {
				download.id = format!("{}/{}", download.id, redownload_id);
			}

			download
		})
	}

	async fn get_download_inner(&self) -> Option<ModDownload> {
		if let Some(github) = &self.github {
			if let Some(github_download) = github.get_download().await {
				return Some(github_download);
			}
		}

		self.latest_version.clone()
	}
}

impl ModGithubInfo {
	// GitHub redirects urls like
	// github.com/{USER}/{PROJECT}/releases/latest
	// to
	// github.com/{USER}/{PROJECT}/releases/tag/{LATEST_TAG}
	// We can use this to cheaply check if a mod is outdated.
	// If we know the asset name stays the same across versions,
	// we can also use this to download the latest version via
	// github.com/{USER}/{PROJECT}/releases/download/{LATEST_TAG}/{ASSET_NAME}
	async fn get_latest_tag(&self) -> Option<String> {
		let url = format!("{}/latest", self.get_releases_url());

		let response = match http_client::get_client_no_redirect()
			.head(&url)
			.send()
			.await
		{
			Ok(response) => Some(response),
			Err(err) => {
				error!("Failed to request head for url `{url}`. Error: {err}");
				None
			}
		}?;

		if response.status().is_redirection() {
			let location_header = response
				.headers()
				// The location header contains the redirect target.
				.get("location")
				.or_else(|| {
					error!("Failed to get location header for url `{url}`");
					None
				})?;

			let location = (match location_header.to_str() {
				Ok(location) => Some(location),
				Err(err) => {
					error!("Failed to parse location header for url `{url}`. Error: {err}");
					None
				}
			})?;

			// Redirect target looks like https://github.com/{USER}/{PROJECT}/releases/tag/{LATEST_TAG}.
			// We only care about that final {LATEST_TAG} part.
			let (_, tag) = regex_captures!(r".*/tag/(.+)", location).or_else(|| {
				error!("Failed to get tag from regex on location `{location}` and url `{url}`.");
				None
			})?;

			Some(tag.to_string())
		} else {
			error!(
				"Expected redirect, but it didn't. Url is `{}` and status was {}.",
				url,
				response.status().as_str()
			);
			None
		}
	}

	pub async fn get_download(&self) -> Option<ModDownload> {
		self.get_latest_tag().await.map(|latest_tag| ModDownload {
			id: latest_tag.clone(),
			root: self.root.clone(),
			runnable: self.runnable.clone(),
			url: format!(
				"{}/download/{}/{}",
				self.get_releases_url(),
				latest_tag,
				self.asset_name
			),
		})
	}

	fn get_releases_url(&self) -> String {
		format!("https://github.com/{}/{}/releases", self.user, self.repo)
	}
}
