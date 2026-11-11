use std::{collections::HashMap, hash::Hash};

use rai_pal_proc_macros::serde_enum;

use crate::{
	architecture::Architecture,
	game_engines::{game_engine::EngineBrand, unity::UnityBackend},
	game_providers::game_provider::GameProviderId,
	game_tag::GameTag,
	operating_system::OperatingSystem,
};

#[serde_enum]
pub enum InstallState {
	Installed,
	NotInstalled,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct FilterItem {
	pub enabled: bool,
	pub locked: bool,
}

impl FilterItem {
	const fn default_item() -> Self {
		Self {
			enabled: true,
			locked: false,
		}
	}
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FilterAction {
	Toggle(bool),
	Lock,
	Only,
}

#[derive(
	Clone, Copy, Debug, PartialEq, Eq, Hash, strum::Display, strum::EnumString, strum::EnumIter,
)]
#[strum(serialize_all = "camelCase")]
pub enum FilterGroupKey {
	Installed,
	Engines,
	ModFamilies,
	Tags,
	Providers,
	UnityBackends,
	ExecutableOs,
	SupportedOs,
	Architectures,
}

impl serde::Serialize for FilterGroupKey {
	fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
		serializer.serialize_str(&self.to_string())
	}
}

impl<'de> serde::Deserialize<'de> for FilterGroupKey {
	fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
		let value = String::deserialize(deserializer)?;

		value
			.parse::<Self>()
			.map_err(|err| serde::de::Error::custom(err))
	}
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct FilterGroup<T: Hash + Eq> {
	pub known: HashMap<T, FilterItem>,
	pub unknown: Option<FilterItem>,
}

impl<T: Hash + Eq> Default for FilterGroup<T> {
	fn default() -> Self {
		Self {
			known: HashMap::new(),
			unknown: None,
		}
	}
}

impl<T: Hash + Eq> FilterGroup<T> {
	pub fn item(&self, key: Option<&T>) -> FilterItem {
		key.map_or_else(
			|| {
				self.unknown
					.clone()
					.unwrap_or_else(FilterItem::default_item)
			},
			|key| {
				self.known
					.get(key)
					.cloned()
					.unwrap_or_else(FilterItem::default_item)
			},
		)
	}

	pub fn has_locked(&self) -> bool {
		self.known.values().any(|item| item.locked)
			|| self.unknown.as_ref().is_some_and(|item| item.locked)
	}

	pub fn has_any_unlocked_enabled(&self, possible_values: &[T], has_unknown: bool) -> bool {
		possible_values.iter().any(|value| {
			let item = self.item(Some(value));
			item.enabled && !item.locked
		}) || (has_unknown && {
			let item = self.item(None);
			item.enabled && !item.locked
		})
	}

	pub fn has_changes(&self, possible_values: &[T], has_unknown: bool) -> bool {
		possible_values.iter().any(|value| {
			let item = self.item(Some(value));
			!item.enabled && !item.locked
		}) || (has_unknown && {
			let item = self.item(None);
			!item.enabled && !item.locked
		})
	}
}

impl<T: Hash + Eq + Clone> FilterGroup<T> {
	pub fn set_item(&mut self, key: Option<T>, item: FilterItem) {
		if key.is_none() && self.unknown.is_none() && item.enabled && !item.locked {
			return;
		}

		if item.enabled && !item.locked {
			if let Some(key) = key {
				self.known.remove(&key);
			} else {
				self.unknown = None;
			}
		} else if let Some(key) = key {
			self.known.insert(key, item);
		} else {
			self.unknown = Some(item);
		}
	}

	pub fn keep_only_locked(&mut self) {
		self.known.retain(|_, item| item.locked);

		if self.unknown.as_ref().is_some_and(|item| !item.locked) {
			self.unknown = None;
		}
	}

	pub fn apply_action(
		&mut self,
		target: Option<T>,
		possible_values: &[T],
		has_unknown: bool,
		action: FilterAction,
	) {
		match action {
			FilterAction::Toggle(enabled) => {
				let mut item = self.item(target.as_ref());
				item.enabled = enabled;
				self.set_item(target, item);

				if !self.has_any_unlocked_enabled(possible_values, has_unknown) {
					self.keep_only_locked();
				}
			}
			FilterAction::Lock => {
				let mut item = self.item(target.as_ref());
				item.locked = !item.locked;
				self.set_item(target, item);
			}
			FilterAction::Only => {
				for value in possible_values {
					let mut item = self.item(Some(value));

					if target.as_ref() == Some(value) {
						item.enabled = true;
					} else if !item.locked {
						item = FilterItem {
							enabled: false,
							locked: false,
						};
					}

					self.set_item(Some(value.clone()), item);
				}

				if has_unknown {
					let mut item = self.item(None);

					if target.is_none() {
						item.enabled = true;
					} else if !item.locked {
						item = FilterItem {
							enabled: false,
							locked: false,
						};
					}

					self.set_item(None, item);
				}
			}
		}
	}
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Default)]
pub struct GamesFilter {
	pub providers: FilterGroup<GameProviderId>,
	pub tags: FilterGroup<GameTag>,
	pub architectures: FilterGroup<Architecture>,
	pub unity_backends: FilterGroup<UnityBackend>,
	pub engines: FilterGroup<EngineBrand>,
	pub executable_os: FilterGroup<OperatingSystem>,
	pub supported_os: FilterGroup<OperatingSystem>,
	pub installed: FilterGroup<InstallState>,
	pub mod_families: FilterGroup<String>,
}

#[serde_enum]
#[derive(Default)]
pub enum GamesSortBy {
	#[default]
	Title,
	Engine,
	ReleaseDate,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct GamesQuery {
	pub filter: GamesFilter,
	pub search: String,
	pub sort_by: GamesSortBy,
	pub sort_descending: bool,
}
