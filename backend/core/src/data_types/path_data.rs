use std::path::PathBuf;

#[derive(serde::Serialize, specta::Type, Clone)]
pub struct PathData(pub PathBuf);

impl rusqlite::types::FromSql for PathData {
	fn column_result(value: rusqlite::types::ValueRef<'_>) -> rusqlite::types::FromSqlResult<Self> {
		Ok(PathData(
			paths_as_strings::decode_path(value.as_str()?).unwrap(),
		)) // TODO error
	}
}

impl rusqlite::types::ToSql for PathData {
	fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput<'_>> {
		Ok(paths_as_strings::encode_path(&self.0).to_string().into())
	}
}
