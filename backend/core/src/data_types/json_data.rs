#[derive(serde::Serialize, specta::Type, Clone)]
pub struct JsonData<T>(pub T);

impl<T> rusqlite::types::FromSql for JsonData<T>
where
	T: serde::de::DeserializeOwned + Eq,
{
	fn column_result(value: rusqlite::types::ValueRef<'_>) -> rusqlite::types::FromSqlResult<Self> {
		let json_str = value.as_str()?;
		let set: T = serde_json::from_str(&json_str).unwrap(); // TODO error
		Ok(JsonData(set))
	}
}

impl<T> rusqlite::types::ToSql for JsonData<T>
where
	T: serde::Serialize + Eq,
{
	fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput<'_>> {
		let json_str = serde_json::to_string(&self.0).unwrap(); // TODO error
		Ok(json_str.into())
	}
}
