use sqlx::{
	Database, Encode, Sqlite,
	encode::IsNull,
	sqlite::{SqliteTypeInfo, SqliteValueRef},
};

#[derive(sqlx::FromRow, serde::Serialize, specta::Type, Clone)]
pub struct JsonData<T>(pub T);

impl<T> sqlx::Decode<'_, sqlx::Sqlite> for JsonData<T>
where
	T: serde::de::DeserializeOwned + Eq,
{
	fn decode(value: SqliteValueRef<'_>) -> std::result::Result<Self, sqlx::error::BoxDynError> {
		let json_str = <&str as sqlx::Decode<sqlx::Sqlite>>::decode(value)?;
		let set: T = serde_json::from_str(&json_str)?;
		Ok(JsonData(set))
	}
}

impl<T> sqlx::Encode<'_, sqlx::Sqlite> for JsonData<T>
where
	T: serde::Serialize + Eq,
{
	fn encode_by_ref(
		&self,
		buf: &mut <Sqlite as Database>::ArgumentBuffer<'_>,
	) -> std::result::Result<IsNull, Box<dyn std::error::Error + Send + Sync>> {
		let json_str = serde_json::to_string(&self.0)?;
		<String as Encode<Sqlite>>::encode_by_ref(&json_str, buf)
	}
}

impl<T> sqlx::Type<Sqlite> for JsonData<T> {
	fn type_info() -> SqliteTypeInfo {
		<String as sqlx::Type<Sqlite>>::type_info()
	}
}
