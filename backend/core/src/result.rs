use std::{env, num, path::PathBuf, result};

use lazy_regex::regex;

#[derive(Debug, thiserror::Error, specta::Type)]
pub enum Error {
	#[error(transparent)]
	Io(
		#[serde(skip)]
		#[from]
		std::io::Error,
	),

	#[error(transparent)]
	Reqwest(
		#[from]
		#[serde(skip)]
		reqwest::Error,
	),

	#[error(transparent)]
	Zip(
		#[from]
		#[serde(skip)]
		zip::result::ZipError,
	),

	#[error(transparent)]
	Json(
		#[from]
		#[serde(skip)]
		serde_json::Error,
	),

	#[error(transparent)]
	ChronoParse(
		#[from]
		#[serde(skip)]
		chrono::ParseError,
	),

	#[error(transparent)]
	Rusql(
		#[from]
		#[serde(skip)]
		rusqlite::Error,
	),

	#[error(transparent)]
	Env(
		#[from]
		#[serde(skip)]
		env::VarError,
	),

	#[error(transparent)]
	UrlEncode(
		#[from]
		#[serde(skip)]
		serde_urlencoded::ser::Error,
	),

	#[error(transparent)]
	HeaderToStr(
		#[from]
		#[serde(skip)]
		reqwest::header::ToStrError,
	),

	#[error(transparent)]
	TryFromInt(
		#[from]
		#[serde(skip)]
		num::TryFromIntError,
	),

	#[error(transparent)]
	Regex(
		#[from]
		#[serde(skip)]
		regex::Error,
	),

	#[error(transparent)]
	SteamLocate(
		#[from]
		#[serde(skip)]
		steamlocate::error::Error,
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
	AppInfoNotFound(String),

	#[error("Failed to retrieve Unity version from asset `{0}`")]
	FailedToParseUnityVersionAsset(PathBuf),

	#[error(
		"Failed to install mod, because the known game information is insufficient. Missing information: `{0}`. Game: `{1}`"
	)]
	ModInstallInfoInsufficient(String, String),

	#[error("Failed to get game data from path `{0}`")]
	FailedToGetGameFromPath(PathBuf),

	#[error("This game has already been added before: `{0}`")]
	GameAlreadyAdded(PathBuf),

	#[error("Data entry not found: `{0}`")]
	DataEntryNotFound(String),

	#[error("Unity backend not known for mod `{0}`")]
	UnityBackendUnknown(String),

	#[error("Download not available for mod `{0}`")]
	ModDownloadNotAvailable(String),

	#[error(
		"Operation can't be completed without a `runnable` section in the mod manifest (rai-pal-manifest.json) `{0}`"
	)]
	RunnableManifestNotFound(String),

	#[error("Can't run mod with ID `{0}` because it isn't a runnable mod.")]
	CantRunNonRunnable(String),

	#[error(
		"Provider ID {0} is invalid for this action, or not supported in the current platform."
	)]
	InvalidProviderId(String),

	#[error(
		"This operation requires game `{0}` to be installed, but the installed game wasn't found."
	)]
	GameNotInstalled(String),
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
