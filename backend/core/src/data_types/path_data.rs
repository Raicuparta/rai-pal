use std::path::PathBuf;

use rusqlite::types::{FromSqlError, FromSqlResult, ToSqlOutput, ValueRef};

#[derive(serde::Serialize, specta::Type, Clone)]
pub struct PathData(pub PathBuf);

impl rusqlite::types::FromSql for PathData {
	fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
		Ok(PathData(
			paths_as_strings::decode_path(value.as_str()?)
				.map_err(|err| FromSqlError::Other(err.into()))?,
		))
	}
}

impl rusqlite::types::ToSql for PathData {
	fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
		Ok(paths_as_strings::encode_path(&self.0).to_string().into())
	}
}
