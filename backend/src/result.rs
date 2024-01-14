use std::{
	env,
	path::PathBuf,
	result,
};

#[derive(Debug, thiserror::Error)]
pub enum Error {
	#[error("Not implemented")]
	NotImplemented,

	#[error(transparent)]
	Io(#[from] std::io::Error),

	#[error(transparent)]
	GlobPattern(#[from] glob::PatternError),

	#[error(transparent)]
	Glob(#[from] glob::GlobError),

	#[error(transparent)]
	Reqwest(#[from] reqwest::Error),

	#[error(transparent)]
	Goblin(#[from] goblin::error::Error),

	#[error("Failed to find Steam. **Is Steam installed**? ({0})")]
	SteamLocate(#[from] steamlocate::Error),

	#[error(transparent)]
	Zip(#[from] zip::result::ZipError),

	#[error(transparent)]
	Tauri(#[from] tauri::Error),

	#[error(transparent)]
	Json(#[from] serde_json::Error),

	#[error(transparent)]
	ChronoParse(#[from] chrono::ParseError),

	#[error(transparent)]
	SQLite(#[from] rusqlite::Error),

	#[error(transparent)]
	Env(#[from] env::VarError),

	#[error("Invalid type `{0}` in binary vdf key/value pair")]
	InvalidBinaryVdfType(u8),

	#[error("Failed to find Rai Pal resources folder")]
	ResourcesNotFound(),

	#[error("Failed to find Rai Pal app data folder")]
	AppDataNotFound(),

	#[error("Failed to parse path (possibly because is a non-UTF-8 string) `{0}`")]
	PathParseFailure(PathBuf),

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

	#[error("Failed to install mod, because the known game information is insufficient. Missing information: `{0}`. Game: `{1}`")]
	ModInstallInfoInsufficient(String, PathBuf),

	#[error("State data is empty")]
	EmptyStateData(),

	#[error("Failed to access state data: `{0}`")]
	FailedToAccessStateData(String),

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

	#[error("Operation can't be completed without a `runnable` section in the mod manifest (rai-pal-manifest.json) `{0}`")]
	RunnableManifestNotFound(String),

	#[error("Can't run command because it isn't defined for this game.")]
	CommandNotDefined(),
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
