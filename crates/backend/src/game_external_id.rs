use std::{
	collections::{BTreeMap, BTreeSet},
	fmt,
	str::FromStr,
};

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::game_providers::game_provider::GameProviderId;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum GameIdKind {
	Provider(GameProviderId),
	NormalizedTitle,
	NormalizedExecutable,
}

impl From<GameProviderId> for GameIdKind {
	fn from(provider_id: GameProviderId) -> Self {
		Self::Provider(provider_id)
	}
}

impl fmt::Display for GameIdKind {
	fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::Provider(provider_id) => write!(formatter, "{provider_id}"),
			Self::NormalizedTitle => formatter.write_str("NormalizedTitle"),
			Self::NormalizedExecutable => formatter.write_str("NormalizedExecutable"),
		}
	}
}

impl FromStr for GameIdKind {
	type Err = strum::ParseError;

	fn from_str(value: &str) -> Result<Self, Self::Err> {
		match value {
			"NormalizedTitle" => Ok(Self::NormalizedTitle),
			"NormalizedExecutable" => Ok(Self::NormalizedExecutable),
			_ => value.parse().map(Self::Provider),
		}
	}
}

impl Serialize for GameIdKind {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		serializer.serialize_str(&self.to_string())
	}
}

impl<'de> Deserialize<'de> for GameIdKind {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		String::deserialize(deserializer)?
			.parse()
			.map_err(serde::de::Error::custom)
	}
}

impl rusqlite::types::FromSql for GameIdKind {
	fn column_result(value: rusqlite::types::ValueRef<'_>) -> rusqlite::types::FromSqlResult<Self> {
		value
			.as_str()?
			.parse()
			.map_err(|error| rusqlite::types::FromSqlError::Other(Box::new(error)))
	}
}

impl rusqlite::types::ToSql for GameIdKind {
	fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput<'_>> {
		Ok(self.to_string().into())
	}
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ExternalIds {
	pub map: BTreeMap<GameIdKind, BTreeSet<String>>,
}

impl ExternalIds {
	pub fn insert(&mut self, kind: impl Into<GameIdKind>, id: impl Into<String>) {
		self.map.entry(kind.into()).or_default().insert(id.into());
	}

	pub fn insert_all(
		&mut self,
		kind: impl Into<GameIdKind>,
		ids: impl IntoIterator<Item = String>,
	) {
		self.map.entry(kind.into()).or_default().extend(ids);
	}

	pub fn first(&self, kind: impl Into<GameIdKind>) -> Option<&str> {
		self.map
			.get(&kind.into())
			.and_then(|ids| ids.first())
			.map(String::as_str)
	}
}
