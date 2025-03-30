use std::{
	collections::{HashMap, HashSet},
	fs,
	path::{Path, PathBuf},
};

use rai_pal_proc_macros::serializable_struct;
use sqlx::{
	Sqlite,
	sqlite::{SqliteTypeInfo, SqliteValueRef},
};

use crate::{
	game_engines::{
		game_engine::{EngineBrand, GameEngine},
		unity::UnityScriptingBackend,
	},
	game_executable::Architecture,
	game_subscription::GameSubscription,
	game_tag::GameTag,
	game_title::GameTitle,
	installed_game::{InstalledGame, InstalledModVersions},
	mod_manifest, paths,
	providers::{
		provider::ProviderId,
		provider_command::{ProviderCommand, ProviderCommandAction},
	},
	remote_game::RemoteGame,
	result::{Error, Result},
};

pub type Map = HashMap<String, Game>;

#[serializable_struct]
#[derive(sqlx::Type, sqlx::FromRow)]
pub struct GameId {
	pub provider_id: ProviderId,
	pub game_id: String,
}

#[derive(sqlx::FromRow, serde::Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct DbGame {
	pub provider_id: ProviderId,
	pub game_id: String,
	pub external_id: String,
	pub display_title: String,
	pub title_discriminator: Option<String>,
	pub normalized_titles: JsonData<Vec<String>>,
	pub thumbnail_url: Option<String>,
	pub release_date: Option<i64>,
	pub exe_path: Option<String>, // TODO convert to Path and back
	pub engine_brand: Option<EngineBrand>,
	pub engine_version: Option<String>,
	pub unity_backend: Option<UnityScriptingBackend>,
	pub architecture: Option<Architecture>,
	pub tags: JsonData<Vec<GameTag>>,
	pub provider_commands: JsonData<HashMap<ProviderCommandAction, ProviderCommand>>,
}

#[derive(sqlx::FromRow, serde::Serialize, specta::Type)]
pub struct JsonData<T>(pub T);

impl<T> sqlx::Decode<'_, sqlx::Sqlite> for JsonData<T>
where
	T: serde::de::DeserializeOwned + Eq,
{
	fn decode(value: SqliteValueRef<'_>) -> std::result::Result<Self, sqlx::error::BoxDynError> {
		let json_str = <&str as sqlx::Decode<sqlx::Sqlite>>::decode(value)?;
		let set: T = serde_json::from_str(&json_str)?;
		Ok(JsonData(set))
	}
}

impl<T> sqlx::Type<Sqlite> for JsonData<T> {
	fn type_info() -> SqliteTypeInfo {
		<String as sqlx::Type<Sqlite>>::type_info()
	}
}

#[serializable_struct]
pub struct Game {
	// ID used to uniquely identify this game in Rai Pal.
	pub id: GameId,

	// ID used to find this game in provider APIs and stuff.
	pub external_id: String,

	pub tags: HashSet<GameTag>,
	pub installed_game: Option<InstalledGame>,
	pub remote_game: Option<RemoteGame>,
	pub title: GameTitle,
	pub thumbnail_url: Option<String>,
	pub release_date: Option<i64>,
	pub provider_commands: HashMap<ProviderCommandAction, ProviderCommand>,
	pub from_subscriptions: HashSet<GameSubscription>,
}

impl Game {
	pub fn new(id: GameId, title: &str) -> Self {
		let title = GameTitle::new(title);
		let mut tags = HashSet::default();
		if title.is_probably_demo() {
			tags.insert(GameTag::Demo);
		}

		Self {
			// We presume the provided ID is also the external ID, but that can be changed after creation.
			external_id: id.game_id.clone(),
			id,
			tags,
			installed_game: None,
			remote_game: None,
			title,
			thumbnail_url: None,
			release_date: None,
			provider_commands: HashMap::default(),
			from_subscriptions: HashSet::default(),
		}
	}

	pub fn add_tag(&mut self, tag: GameTag) -> &mut Self {
		self.tags.insert(tag);
		self
	}

	pub fn set_thumbnail_url(&mut self, thumbnail_url: &str) -> &mut Self {
		self.thumbnail_url = Some(thumbnail_url.to_string());
		self
	}

	pub fn set_release_date(&mut self, release_date: i64) -> &mut Self {
		self.release_date = Some(release_date);
		self
	}

	pub fn add_provider_command(
		&mut self,
		command_action: ProviderCommandAction,
		command: ProviderCommand,
	) -> &mut Self {
		self.provider_commands.insert(command_action, command);
		self
	}

	pub const fn get_engine(&self) -> Option<&GameEngine> {
		if let Some(installed_game) = &self.installed_game {
			if let Some(engine) = installed_game.executable.engine.as_ref() {
				return Some(engine);
			}
		}

		if let Some(remote_game) = &self.remote_game {
			return remote_game.engine.as_ref();
		}

		None
	}

	pub fn try_get_installed_game(&self) -> Result<&InstalledGame> {
		self.installed_game
			.as_ref()
			.ok_or_else(|| Error::GameNotInstalled(self.title.display.clone()))
	}

	pub fn try_get_installed_game_mut(&mut self) -> Result<&mut InstalledGame> {
		self.installed_game
			.as_mut()
			.ok_or_else(|| Error::GameNotInstalled(self.title.display.clone()))
	}
}

impl DbGame {
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

	pub fn get_installed_mod_versions(&self) -> InstalledModVersions {
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

	pub fn try_get_exe_path(&self) -> Result<PathBuf> {
		Ok(PathBuf::from(self.exe_path.as_ref().ok_or_else(|| {
			Error::GameNotInstalled(self.display_title.clone())
		})?))
	}
}
