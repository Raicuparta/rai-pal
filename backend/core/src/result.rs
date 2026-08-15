use std::{
	env,
	fmt,
	num,
	path::PathBuf,
	result,
	time::SystemTimeError,
};

use lazy_regex::regex;

use crate::{
	game_engines::game_engine::EngineBrand,
	game_providers::game_provider::GameProviderId,
};

#[derive(Debug, thiserror::Error, specta::Type)]
pub enum Error {
	#[error(transparent)]
	Io(
		#[specta(skip)]
		#[from]
		std::io::Error,
	),

	#[error(transparent)]
	Reqwest(
		#[from]
		#[specta(skip)]
		reqwest::Error,
	),

	#[error(transparent)]
	Zip(
		#[from]
		#[specta(skip)]
		zip::result::ZipError,
	),

	#[error(transparent)]
	Json(
		#[from]
		#[specta(skip)]
		serde_json::Error,
	),

	#[error(transparent)]
	ChronoParse(
		#[from]
		#[specta(skip)]
		chrono::ParseError,
	),

	#[error(transparent)]
	Rusql(
		#[from]
		#[specta(skip)]
		rusqlite::Error,
	),

	#[error(transparent)]
	Env(
		#[from]
		#[specta(skip)]
		env::VarError,
	),

	#[error(transparent)]
	UrlEncode(
		#[from]
		#[specta(skip)]
		serde_urlencoded::ser::Error,
	),

	#[error(transparent)]
	HeaderToStr(
		#[from]
		#[specta(skip)]
		reqwest::header::ToStrError,
	),

	#[error(transparent)]
	TryFromInt(
		#[from]
		#[specta(skip)]
		num::TryFromIntError,
	),

	#[error(transparent)]
	Regex(
		#[from]
		#[specta(skip)]
		regex::Error,
	),

	#[error(transparent)]
	SteamLocate(
		#[from]
		#[specta(skip)]
		steamlocate::error::Error,
	),

	#[error(transparent)]
	SystemTime(
		#[from]
		#[specta(skip)]
		SystemTimeError,
	),

	#[error(transparent)]
	ParseInt(
		#[from]
		#[specta(skip)]
		num::ParseIntError,
	),

	#[error(transparent)]
	Utf8(
		#[from]
		#[specta(skip)]
		std::str::Utf8Error,
	),

	#[error("Invalid type `{0}` in binary vdf for key {1}")]
	InvalidBinaryVdfType(u8, String),

	#[error("Failed to find app data folder")]
	AppDataNotFound(),

	#[error("Failed to parse path (possibly because is a non-UTF-8 string) `{0}`")]
	InvalidOsStr(String),

	#[error("Failed to get folder parent for path `{0}`")]
	PathParentNotFound(PathBuf),

	#[error("Tried to read empty file `{0}`")]
	EmptyFile(PathBuf),

	#[error(
		"Failed to find Steam cache file. **Try restarting Steam**. (Tried to read from `{0}`)"
	)]
	SteamAppInfoNotFound(PathBuf),

	#[error("Steam Proton handling error: {0}")]
	SteamProton(String),

	#[error("Itch handling error: {0}")]
	Itch(String),

	#[error("Failed to retrieve Unity version from asset `{0}`")]
	FailedToParseUnityVersionAsset(PathBuf),

	#[error(
		"Failed to install mod, because the known game information is insufficient. Missing information: `{0}`. Game: `{1}`"
	)]
	ModInstallInfoInsufficient(String, String),

	#[error("Local mod manifest in `{0}` defines 'download' section, makes no sense.")]
	LocalModCantDefineDownload(String),

	#[error("Failed to get game data from path `{0}`")]
	FailedToGetGameFromPath(PathBuf),

	#[error("This game has already been added before: `{0}`")]
	GameAlreadyAdded(PathBuf),

	#[error("Data entry not found: `{0}`")]
	DataEntryNotFound(String),

	#[error("Unity backend not known for mod `{0}`")]
	UnityBackendUnknown(String),

	#[error(
		"Provider ID {0} is invalid for this action, or not supported in the current platform."
	)]
	InvalidProviderId(String),

	#[error(
		"This operation requires game `{0}` to be installed, but the installed game wasn't found."
	)]
	GameNotInstalled(String),

	#[error("This operation requires mod `{0}` to be installed.")]
	ModNotInstalled(String),

	#[error("Failed to find game executable at `{0}`")]
	NoExecutableFound(PathBuf),

	#[error("Failed to acquire lock for database: `{0}`")]
	DatabaseLockFailed(String),

	#[error("Auth failed: `{0}`")]
	Auth(String),

	#[error("Failed to find mod manifest in path: `{0}`")]
	ManifestNotFound(String),

	#[error("Provider {0} doesn't support this operation: `{1}`")]
	UnsupportedProviderOperation(GameProviderId, String),

	#[error("Game engine {0} doesn't support this operation: `{1}`")]
	UnsupportedEngineOperation(EngineBrand, String),

	#[error("Game engine detection missing for this operation: `{0}`")]
	OperationRequiresEngine(String),

	#[error("Required information for mod with ID `{0}` is missing. Expected `{1}`")]
	ModInfoMissing(String, String),

	#[error("No command fount for opening `{0}`")]
	NoCommandForOpen(String),

	#[error("Can't do this without a game. If possible, try it from the games tab instead.")]
	GameNeeded(),

	#[error("Wine prefix not initialized at `{0}`. Please start the game at least once first.")]
	WinePrefixNotInitialized(PathBuf),
}

impl serde::Serialize for Error {
	fn serialize<S>(&self, serializer: S) -> result::Result<S::Ok, S::Error>
	where
		S: serde::ser::Serializer,
	{
		serializer.serialize_str(self.to_string().as_ref())
	}
}

pub type Result<T = ()> = result::Result<T, Error>;

pub trait LogErrExt<T, E> {
	fn ok_or_log(self, message: &str) -> Option<T>;
}

impl<T, E> LogErrExt<T, E> for result::Result<T, E>
where
	E: fmt::Display,
{
	fn ok_or_log(self, message: &str) -> Option<T> {
		match self {
			Ok(val) => Some(val),
			Err(err) => {
				log::warn!("{message}: {err}");
				None
			}
		}
	}
}
