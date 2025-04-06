use std::{
	collections::HashMap,
	fs,
	path::{Path, PathBuf},
};

use rai_pal_proc_macros::serializable_struct;
use sqlx::Sqlite;

use crate::{
	data_types::{json_data::JsonData, path_data::PathData},
	game_engines::{game_engine::EngineBrand, unity::UnityScriptingBackend},
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
#[derive(sqlx::Type, sqlx::FromRow)]
pub struct GameId {
	pub provider_id: ProviderId,
	pub game_id: String,
}

#[derive(sqlx::FromRow, serde::Serialize, specta::Type, Clone)]
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
	pub engine_version: Option<String>,
	pub unity_backend: Option<UnityScriptingBackend>,
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
			engine_version: None,
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
		self.engine_version = executable
			.engine
			.as_ref()
			.and_then(|e| e.version.as_ref().map(|v| v.display.clone()));
		self.architecture = executable.architecture;
		self.unity_backend = executable.scripting_backend;
		self.add_provider_command(
			ProviderCommandAction::StartViaExe,
			ProviderCommand::Path(executable.path.clone(), Vec::default()),
		);
		self
	}
}

pub trait InsertGame {
	async fn insert_game(&self, game: &DbGame) -> Result;
}

impl InsertGame for sqlx::Pool<Sqlite> {
	async fn insert_game(&self, game: &DbGame) -> Result {
		let mut transaction = self.begin().await?;

		sqlx::query::<Sqlite>(
			"INSERT OR REPLACE INTO games (
					provider_id,
					game_id,
					external_id,
					display_title,
					thumbnail_url,
					release_date,
					tags,
					title_discriminator,
					provider_commands
				) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
		)
		.bind(game.provider_id)
		.bind(game.game_id.clone())
		.bind(game.external_id.clone())
		.bind(game.display_title.clone())
		.bind(game.thumbnail_url.clone())
		.bind(game.release_date)
		.bind(game.tags.clone())
		.bind(game.title_discriminator.clone())
		.bind(game.provider_commands.clone())
		.execute(&mut *transaction)
		.await?;

		if let Some(exe_path) = game.exe_path.as_ref() {
			sqlx::query::<Sqlite>(
					"INSERT OR REPLACE INTO installed_games (provider_id, game_id, exe_path, engine_brand, engine_version, unity_backend, architecture)
					 VALUES ($1, $2, $3, $4, $5, $6, $7)"
				)
					.bind(game.provider_id)
					.bind(game.game_id.clone())
					.bind(exe_path)
					.bind(game.engine_brand)
					.bind(game.engine_version.clone())
					.bind(game.unity_backend.clone())
					.bind(game.architecture.clone()
				).execute(&mut *transaction).await?;
		}

		for normalized_title in get_normalized_titles(&game.display_title) {
			sqlx::query::<Sqlite>(
				"INSERT OR REPLACE INTO normalized_titles (provider_id, game_id, normalized_title)
							VALUES ($1, $2, $3)",
			)
			.bind(game.provider_id)
			.bind(game.game_id.clone())
			.bind(normalized_title.clone())
			.execute(&mut *transaction)
			.await?;
		}

		transaction.commit().await?; // TODO roll back if needed

		Ok(())
	}
}
