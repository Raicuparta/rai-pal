use std::{
	env,
	fmt,
	num,
	path::PathBuf,
	result,
	time::SystemTimeError,
};

use lazy_regex::regex;
use strum::Display;

#[derive(Debug, Display, specta::Type)]
pub enum Error {
	Io,

	Reqwest,

	Zip,

	Json,

	ChronoParse,

	Rusql,

	Env,

	UrlEncode,

	HeaderToStr,

	TryFromInt,

	Regex,

	SteamLocate,

	SystemTime,

	InvalidBinaryVdfType(u8, String),

	AppDataNotFound,

	InvalidOsStr(String),

	PathParentNotFound(PathBuf),

	EmptyFile(PathBuf),

	SteamAppInfoNotFound(PathBuf),

	FailedToParseUnityVersionAsset(PathBuf),

	ModInstallInfoInsufficient(String, String),

	FailedToGetGameFromPath(PathBuf),

	GameAlreadyAdded(PathBuf),

	DataEntryNotFound(String),

	UnityBackendUnknown(String),

	RunnableManifestNotFound(String),

	CantRunNonRunnable(String),

	InvalidProviderId(String),

	GameNotInstalled(String),

	NoExecutableFound(PathBuf),

	DatabaseLockFailed(String),

	DiscordOAuth(String),

	ManifestNotFound(String),

	UnsupportedOperation(String),

	ModInfoMissing(String, String),
}

pub type Result<T = ()> = anyhow::Result<T>;

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
