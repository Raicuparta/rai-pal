use rusqlite::{
	Row,
	RowIndex,
	types::{
		FromSql,
		FromSqlError,
		FromSqlResult,
		ValueRef,
	},
};
use serde::de::DeserializeOwned;

#[derive(serde::Serialize, specta::Type, Clone)]
pub struct JsonData<T>(pub T);

impl<T> rusqlite::types::FromSql for JsonData<T>
where
	T: serde::de::DeserializeOwned + Eq,
{
	fn column_result(value: rusqlite::types::ValueRef<'_>) -> rusqlite::types::FromSqlResult<Self> {
		Ok(Self(
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

struct FromJson<T>(T);

impl<T: DeserializeOwned> FromSql for FromJson<T> {
	fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
		let parsed = serde_json::from_str(value.as_str()?)
			.map_err(|err| FromSqlError::Other(Box::new(err)))?;
		Ok(Self(parsed))
	}
}

pub trait RowExt {
	fn get_json<I: RowIndex, T: DeserializeOwned>(&self, index: I) -> rusqlite::Result<T>;
}

impl RowExt for Row<'_> {
	fn get_json<I: RowIndex, T: DeserializeOwned>(&self, index: I) -> rusqlite::Result<T> {
		self.get::<I, FromJson<T>>(index).map(|wrapper| wrapper.0)
	}
}
