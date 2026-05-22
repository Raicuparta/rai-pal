use std::result;

use anyhow;

#[derive(Debug, thiserror::Error, specta::Type)]
pub enum Error {
	#[error(transparent)]
	Tauri(
		#[specta(skip)]
		#[from]
		tauri::Error,
	),

	#[error(transparent)]
	Io(
		#[specta(skip)]
		#[from]
		std::io::Error,
	),

	#[error(transparent)]
	Rusql(
		#[from]
		#[specta(skip)]
		rusqlite::Error,
	),

	#[error(transparent)]
	SerdeJson(
		#[specta(skip)]
		#[from]
		serde_json::error::Error,
	),

	#[error(transparent)]
	SystemTimeError(
		#[specta(skip)]
		#[from]
		std::time::SystemTimeError,
	),

	#[error("Failed to access state data: `{0}`")]
	FailedToAccessStateData(String),

	#[error("Not supported on current platform. Linux only.")]
	#[allow(dead_code)] // Unused on Linux.
	LinuxOnly(),

	#[error(transparent)]
	Anyhow(
		#[specta(skip)]
		#[from]
		anyhow::Error,
	),
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
