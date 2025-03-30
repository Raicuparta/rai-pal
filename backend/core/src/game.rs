use std::collections::{HashMap, HashSet};

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
	installed_game::InstalledGame,
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
	pub normalized_titles: JsonVec<String>,
	pub thumbnail_url: Option<String>,
	pub release_date: Option<i64>,
	pub exe_path: Option<String>,
	pub engine_brand: Option<EngineBrand>,
	pub engine_version: Option<String>,
	pub unity_backend: Option<UnityScriptingBackend>,
	pub architecture: Option<Architecture>,
	pub tags: JsonVec<GameTag>,
}

#[derive(sqlx::FromRow, serde::Serialize, specta::Type)]
pub struct JsonVec<T>(pub Vec<T>);

impl<T> sqlx::Decode<'_, sqlx::Sqlite> for JsonVec<T>
where
	T: serde::de::DeserializeOwned + Eq + std::hash::Hash,
{
	fn decode(value: SqliteValueRef<'_>) -> std::result::Result<Self, sqlx::error::BoxDynError> {
		let json_str = <&str as sqlx::Decode<sqlx::Sqlite>>::decode(value)?;
		let set: Vec<T> = serde_json::from_str(&json_str)?;
		Ok(JsonVec(set))
	}
}

impl<T> sqlx::Type<Sqlite> for JsonVec<T> {
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
