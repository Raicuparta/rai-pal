use std::result;

use anyhow;
use strum::Display;

#[derive(Debug, Display, specta::Type)]
pub enum AppError {
	FailedToAccessStateData(String),

	#[allow(dead_code)] // Unused on Linux.
	LinuxOnly(),

	#[specta(skip)]
	Anyhow(anyhow::Error),
}

impl<E> From<E> for AppError
where
	E: Into<anyhow::Error>,
{
	fn from(value: E) -> Self {
		Self::Anyhow(value.into())
	}
}

impl serde::Serialize for AppError {
	fn serialize<S>(&self, serializer: S) -> result::Result<S::Ok, S::Error>
	where
		S: serde::ser::Serializer,
	{
		serializer.serialize_str(self.to_string().as_ref())
	}
}

pub type AppResult<T = ()> = result::Result<T, AppError>;
