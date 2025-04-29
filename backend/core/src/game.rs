use std::{
	collections::HashMap,
	fs,
	path::{Path, PathBuf},
	time::{SystemTime, UNIX_EPOCH},
};

use rai_pal_proc_macros::serializable_struct;

use crate::{
	data_types::{json_data::JsonData, path_data::PathData},
	game_engines::{game_engine::EngineBrand, unity::UnityBackend},
	game_executable::{Architecture, GameExecutable},
	game_tag::GameTag,
	game_title::get_normalized_titles,
	mod_manifest, paths,
	providers::{
		provider::ProviderId,
		provider_command::{ProviderCommand, ProviderCommandAction},
	},
	result::{Error, Result},
};

#[serializable_struct]
pub struct GameId {
	pub provider_id: ProviderId,
	pub game_id: String,
}

#[derive(serde::Serialize, specta::Type, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DbGame {
	pub provider_id: ProviderId,
	pub game_id: String,
	pub external_id: String,
	pub display_title: String,
	pub title_discriminator: Option<String>,
	pub thumbnail_url: Option<String>,
	pub release_date: Option<i64>,
	pub exe_path: Option<PathData>,
	pub engine_brand: Option<EngineBrand>,
	pub engine_version_major: Option<u32>,
	pub engine_version_minor: Option<u32>,
	pub engine_version_patch: Option<u32>,
	pub engine_version_display: Option<String>,
	pub unity_backend: Option<UnityBackend>,
	pub architecture: Option<Architecture>,
	pub tags: JsonData<Vec<GameTag>>,
	pub provider_commands: JsonData<HashMap<ProviderCommandAction, ProviderCommand>>,
}

impl DbGame {
	pub fn new(provider_id: ProviderId, game_id: String, title: String) -> Self {
		Self {
			provider_id,
			external_id: game_id.clone(),
			game_id,
			display_title: title,
			title_discriminator: None,
			thumbnail_url: None,
			release_date: None,
			exe_path: None,
			engine_brand: None,
			engine_version_major: None,
			engine_version_minor: None,
			engine_version_patch: None,
			engine_version_display: None,
			unity_backend: None,
			architecture: None,
			tags: JsonData(Vec::default()),
			provider_commands: JsonData(HashMap::default()),
		}
	}

	pub fn open_game_folder(&self) -> Result {
		paths::open_folder_or_parent(&self.try_get_exe_path()?)
	}

	pub fn open_mods_folder(&self) -> Result {
		paths::open_folder_or_parent(&self.get_installed_mods_folder()?)
	}

	pub fn uninstall_all_mods(&self) -> Result {
		Ok(fs::remove_dir_all(self.get_installed_mods_folder()?)?)
	}

	pub fn get_manifest_paths(&self) -> Vec<PathBuf> {
		match self.get_installed_mod_manifest_path("*") {
			Ok(manifests_path) => {
				if !manifests_path.parent().is_some_and(Path::exists) {
					return Vec::default();
				}
				paths::glob_path(&manifests_path)
			}
			Err(err) => {
				log::error!(
					"Failed to get mod manifests glob path for game {}. Error: {}",
					self.display_title,
					err
				);
				Vec::default()
			}
		}
	}

	pub fn get_installed_mod_versions(&self) -> HashMap<String, String> {
		self.get_manifest_paths()
			.iter()
			.filter_map(|manifest_path| {
				let manifest = mod_manifest::get(manifest_path)?;

				Some((
					manifest_path.file_stem()?.to_str()?.to_string(),
					manifest.version,
				))
			})
			.collect()
	}

	pub fn get_installed_mod_manifest_path(&self, mod_id: &str) -> Result<PathBuf> {
		Ok(self
			.get_installed_mods_folder()?
			.join("manifests")
			.join(format!("{mod_id}.json")))
	}

	pub fn get_installed_mods_folder(&self) -> Result<PathBuf> {
		let installed_mods_folder = paths::app_data_path()?
			.join("installed-mods")
			.join(&paths::hash_path(&self.try_get_exe_path()?));
		fs::create_dir_all(&installed_mods_folder)?;

		Ok(installed_mods_folder)
	}

	pub fn try_get_exe_path(&self) -> Result<&Path> {
		Ok(&self
			.exe_path
			.as_ref()
			.ok_or_else(|| Error::GameNotInstalled(self.display_title.clone()))?
			.0)
	}

	pub fn try_get_exe_name(&self) -> Result<String> {
		let path = self.try_get_exe_path()?;
		Ok(path
			.file_name()
			.and_then(|file_name| file_name.to_str())
			.map(|file_name| file_name.to_string())
			.ok_or_else(|| Error::InvalidOsStr(path.display().to_string()))?)
	}

	pub fn add_provider_command(
		&mut self,
		command_action: ProviderCommandAction,
		command: ProviderCommand,
	) -> &mut Self {
		self.provider_commands.0.insert(command_action, command);
		self
	}

	pub fn add_tag(&mut self, tag: GameTag) -> &mut Self {
		self.tags.0.push(tag);
		self
	}

	pub fn set_executable(&mut self, executable: &GameExecutable) -> &mut Self {
		self.exe_path = Some(PathData(executable.path.clone()));
		self.engine_brand = executable.engine.as_ref().map(|e| e.brand.clone());
		if let Some(engine_version) = executable
			.engine
			.as_ref()
			.and_then(|engine| engine.version.as_ref())
		{
			self.engine_version_major = Some(engine_version.numbers.major);
			self.engine_version_minor = engine_version.numbers.minor;
			self.engine_version_patch = engine_version.numbers.patch;
			self.engine_version_display = Some(engine_version.display.clone());
		}
		self.architecture = executable.architecture;
		self.unity_backend = executable.unity_backend;
		self.add_provider_command(
			ProviderCommandAction::StartViaExe,
			ProviderCommand::Path(executable.path.clone(), Vec::default()),
		);
		self
	}
}

pub trait InsertGame {
	fn insert_game(&self, game: &DbGame) -> Result;
}

impl InsertGame for rusqlite::Connection {
	fn insert_game(&self, game: &DbGame) -> Result {
		// let transaction = self.transaction()?;
		// TODO understand how to do a transaction here

		// TODO prepare this only once since it's always the same.
		self.prepare(
			"INSERT OR REPLACE INTO games (
					provider_id,
					game_id,
					external_id,
					display_title,
					thumbnail_url,
					release_date,
					tags,
					title_discriminator,
					provider_commands,
					created_at
				) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)",
		)?
		.execute(rusqlite::params![
			game.provider_id,
			game.game_id.clone(),
			game.external_id.clone(),
			game.display_title.clone(),
			game.thumbnail_url.clone(),
			game.release_date,
			game.tags.clone(),
			game.title_discriminator.clone(),
			game.provider_commands.clone(),
			SystemTime::now()
				.duration_since(UNIX_EPOCH)
				.unwrap()
				.as_secs()
		])?;

		if let Some(exe_path) = game.exe_path.as_ref() {
			self.prepare(
				"INSERT OR REPLACE INTO installed_games (
						provider_id,
						game_id,
						exe_path,
						engine_brand,
						engine_version_major,
						engine_version_minor,
						engine_version_patch,
						engine_version_display,
						unity_backend,
						architecture
					)
					 VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)",
			)?
			.execute(rusqlite::params![
				game.provider_id,
				game.game_id.clone(),
				exe_path,
				game.engine_brand,
				game.engine_version_major,
				game.engine_version_minor,
				game.engine_version_patch,
				game.engine_version_display.clone(),
				game.unity_backend.clone(),
				game.architecture.clone(),
			])?;
		}

		for normalized_title in get_normalized_titles(&game.display_title) {
			self.prepare(
				"INSERT OR REPLACE INTO normalized_titles (provider_id, game_id, normalized_title)
							VALUES ($1, $2, $3)",
			)?
			.execute(rusqlite::params![
				game.provider_id,
				game.game_id.clone(),
				normalized_title.clone(),
			])?;
		}

		// TODO understand how to do a transaction here
		// transaction.commit()?;

		Ok(())
	}
}
