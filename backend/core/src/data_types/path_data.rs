use std::path::PathBuf;

use sqlx::{
	Database, Encode, Sqlite,
	encode::IsNull,
	sqlite::{SqliteTypeInfo, SqliteValueRef},
};

#[derive(sqlx::FromRow, serde::Serialize, specta::Type, Clone)]
pub struct PathData(pub PathBuf);

impl sqlx::Decode<'_, sqlx::Sqlite> for PathData {
	fn decode(value: SqliteValueRef<'_>) -> std::result::Result<Self, sqlx::error::BoxDynError> {
		Ok(PathData(paths_as_strings::decode_path(
			<&str as sqlx::Decode<sqlx::Sqlite>>::decode(value)?,
		)?))
	}
}

impl sqlx::Encode<'_, sqlx::Sqlite> for PathData {
	fn encode_by_ref(
		&self,
		buf: &mut <Sqlite as Database>::ArgumentBuffer<'_>,
	) -> std::result::Result<IsNull, Box<dyn std::error::Error + Send + Sync>> {
		<String as Encode<Sqlite>>::encode_by_ref(
			&paths_as_strings::encode_path(&self.0).to_string(),
			buf,
		)
	}
}

impl sqlx::Type<Sqlite> for PathData {
	fn type_info() -> SqliteTypeInfo {
		<String as sqlx::Type<Sqlite>>::type_info()
	}
}

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
