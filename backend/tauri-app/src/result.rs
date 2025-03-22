use std::result;

#[derive(Debug, thiserror::Error, specta::Type)]
pub enum Error {
	#[error(transparent)]
	Tauri(
		#[serde(skip)]
		#[from]
		tauri::Error,
	),

	#[error(transparent)]
	Core(
		#[serde(skip)]
		#[from]
		rai_pal_core::result::Error,
	),

	#[error(transparent)]
	Io(
		#[serde(skip)]
		#[from]
		std::io::Error,
	),

	#[error(transparent)]
	SQL(
		#[from]
		#[serde(skip)]
		sqlx::Error,
	),

	#[error(transparent)]
	SerdeJson(
		#[serde(skip)]
		#[from]
		serde_json::error::Error,
	),

	#[error("Failed to get app resources path: `{0}`")]
	FailedToGetResourcesPath(String),

	#[error("Failed to access state data: `{0}`")]
	FailedToAccessStateData(String),
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
