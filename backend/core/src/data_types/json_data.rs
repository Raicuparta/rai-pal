use rusqlite::types::FromSqlError;

#[derive(serde::Serialize, specta::Type, Clone)]
pub struct JsonData<T>(pub T);

impl<T> rusqlite::types::FromSql for JsonData<T>
where
	T: serde::de::DeserializeOwned + Eq,
{
	fn column_result(value: rusqlite::types::ValueRef<'_>) -> rusqlite::types::FromSqlResult<Self> {
		Ok(JsonData(
			serde_json::from_str(value.as_str()?).map_err(|err| FromSqlError::Other(err.into()))?,
		))
	}
}

impl<T> rusqlite::types::ToSql for JsonData<T>
where
	T: serde::Serialize + Eq,
{
	fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput<'_>> {
		Ok(serde_json::to_string(&self.0)
			.map_err(|err| FromSqlError::Other(err.into()))?
			.into())
	}
}
