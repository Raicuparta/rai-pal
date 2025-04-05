use std::{
	collections::HashMap,
	fs,
	path::{Path, PathBuf},
};

use rai_pal_proc_macros::serializable_struct;
use sqlx::{
	Database, Sqlite,
	encode::IsNull,
	sqlite::{SqliteTypeInfo, SqliteValueRef},
};

use crate::{
	game_engines::{game_engine::EngineBrand, unity::UnityScriptingBackend},
	game_executable::Architecture,
	game_tag::GameTag,
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
	pub exe_path: Option<String>, // TODO convert to Path and back
	pub engine_brand: Option<EngineBrand>,
	pub engine_version: Option<String>,
	pub unity_backend: Option<UnityScriptingBackend>,
	pub architecture: Option<Architecture>,
	pub tags: JsonData<Vec<GameTag>>,
	pub provider_commands: JsonData<HashMap<ProviderCommandAction, ProviderCommand>>,
}

#[derive(sqlx::FromRow, serde::Serialize, specta::Type, Clone)]
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

impl<T> sqlx::Encode<'_, sqlx::Sqlite> for JsonData<T>
where
	T: serde::Serialize + Eq,
{
	fn encode_by_ref(
		&self,
		buf: &mut <Sqlite as Database>::ArgumentBuffer<'_>,
	) -> std::result::Result<IsNull, Box<dyn std::error::Error + Send + Sync>> {
		let json_str = serde_json::to_string(&self.0)?;
		<String as sqlx::Encode<sqlx::Sqlite>>::encode_by_ref(&json_str, buf)
	}
}

impl<T> sqlx::Type<Sqlite> for JsonData<T> {
	fn type_info() -> SqliteTypeInfo {
		<String as sqlx::Type<Sqlite>>::type_info()
	}
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

	pub fn try_get_exe_path(&self) -> Result<PathBuf> {
		Ok(PathBuf::from(self.exe_path.as_ref().ok_or_else(|| {
			Error::GameNotInstalled(self.display_title.clone())
		})?))
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
}
