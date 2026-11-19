use std::{
	collections::{BTreeMap, BTreeSet},
	fmt,
	str::FromStr,
};

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::game_providers::game_provider::GameProviderId;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ExternalIdProvider {
	GameProvider(GameProviderId),
	NormalizedTitle,
}

impl From<GameProviderId> for ExternalIdProvider {
	fn from(provider_id: GameProviderId) -> Self {
		Self::GameProvider(provider_id)
	}
}

impl fmt::Display for ExternalIdProvider {
	fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::GameProvider(provider_id) => write!(formatter, "{provider_id}"),
			Self::NormalizedTitle => formatter.write_str("NormalizedTitle"),
		}
	}
}

impl FromStr for ExternalIdProvider {
	type Err = strum::ParseError;

	fn from_str(value: &str) -> Result<Self, Self::Err> {
		if value == "NormalizedTitle" {
			return Ok(Self::NormalizedTitle);
		}

		value.parse().map(Self::GameProvider)
	}
}

impl Serialize for ExternalIdProvider {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		serializer.serialize_str(&self.to_string())
	}
}

impl<'de> Deserialize<'de> for ExternalIdProvider {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		String::deserialize(deserializer)?
			.parse()
			.map_err(serde::de::Error::custom)
	}
}

impl rusqlite::types::FromSql for ExternalIdProvider {
	fn column_result(value: rusqlite::types::ValueRef<'_>) -> rusqlite::types::FromSqlResult<Self> {
		value
			.as_str()?
			.parse()
			.map_err(|error| rusqlite::types::FromSqlError::Other(Box::new(error)))
	}
}

impl rusqlite::types::ToSql for ExternalIdProvider {
	fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput<'_>> {
		Ok(self.to_string().into())
	}
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ExternalIds {
	pub map: BTreeMap<ExternalIdProvider, BTreeSet<String>>,
}

impl ExternalIds {
	pub fn insert(&mut self, provider: impl Into<ExternalIdProvider>, id: impl Into<String>) {
		self.map
			.entry(provider.into())
			.or_default()
			.insert(id.into());
	}

	pub fn insert_all(
		&mut self,
		provider: impl Into<ExternalIdProvider>,
		ids: impl IntoIterator<Item = String>,
	) {
		self.map.entry(provider.into()).or_default().extend(ids);
	}

	pub fn first(&self, provider: impl Into<ExternalIdProvider>) -> Option<&str> {
		self.map
			.get(&provider.into())
			.and_then(|ids| ids.first())
			.map(String::as_str)
	}
}
