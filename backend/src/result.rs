use std::{
	path::PathBuf,
	result,
};

#[derive(Debug, thiserror::Error)]
pub enum Error {
	#[error(transparent)]
	Io(#[from] std::io::Error),

	#[error(transparent)]
	Glob(#[from] glob::PatternError),

	#[error(transparent)]
	Reqwest(#[from] reqwest::Error),

	#[error(transparent)]
	Goblin(#[from] goblin::error::Error),

	#[error("Failed to find Steam. **Is Steam installed**? ({0})")]
	SteamLocate(#[from] steamlocate::Error),

	#[error(transparent)]
	Zip(#[from] zip::result::ZipError),

	#[error("Invalid type `{0}` in binary vdf key/value pair")]
	InvalidBinaryVdfType(u8),

	#[error("Failed to get file name from path `{0}`")]
	FailedToGetFileName(PathBuf),

	#[error("Failed to find game with ID `{0}`")]
	GameNotFound(String),

	#[error("Failed to find mod with ID `{0}`")]
	ModNotFound(String),

	#[error("Failed to find mod loader with ID `{0}`")]
	ModLoaderNotFound(String),

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

	#[error("Failed to create copy of game with ID `{0}`")]
	GameCopyFailed(String),

	#[error(
		"Failed to find Steam cache file. **Try restarting Steam**. (Tried to read from `{0}`)"
	)]
	AppInfoNotFound(String),

	#[error("Failed to retrieve Unity version from asset `{0}`")]
	FailedToParseUnityVersionAsset(PathBuf),

	#[error("Can't install a mod without knowing the Unity scripting backend. Game: `{0}`")]
	InstallModWithoutBackend(PathBuf),
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
