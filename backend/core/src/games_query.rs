use std::{collections::HashMap, hash::Hash};

use rai_pal_proc_macros::{serializable_enum, serializable_struct};

use crate::{
	architecture::Architecture,
	game_engines::{game_engine::EngineBrand, unity::UnityBackend},
	game_providers::game_provider::GameProviderId,
	game_tag::GameTag,
	operating_system::OperatingSystem,
};

#[serializable_enum]
pub enum InstallState {
	Installed,
	NotInstalled,
}

#[serializable_struct]
pub struct FilterItem {
	pub enabled: bool,
	pub locked: bool,
}

impl FilterItem {
	fn default_item() -> Self {
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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

impl FilterGroupKey {
	pub const ALL: [Self; 9] = [
		Self::Installed,
		Self::Engines,
		Self::ModFamilies,
		Self::Tags,
		Self::Providers,
		Self::UnityBackends,
		Self::ExecutableOs,
		Self::SupportedOs,
		Self::Architectures,
	];

	pub fn as_str(self) -> &'static str {
		match self {
			Self::Installed => "installed",
			Self::Engines => "engines",
			Self::ModFamilies => "modFamilies",
			Self::Tags => "tags",
			Self::Providers => "providers",
			Self::UnityBackends => "unityBackends",
			Self::ExecutableOs => "executableOs",
			Self::SupportedOs => "supportedOs",
			Self::Architectures => "architectures",
		}
	}
}

impl std::str::FromStr for FilterGroupKey {
	type Err = ();

	fn from_str(value: &str) -> Result<Self, Self::Err> {
		Self::ALL
			.into_iter()
			.find(|key| key.as_str() == value)
			.ok_or(())
	}
}

#[derive(serde::Serialize, serde::Deserialize, specta::Type, Clone, Debug)]
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
		match key {
			Some(key) => self
				.known
				.get(key)
				.cloned()
				.unwrap_or_else(FilterItem::default_item),
			None => self
				.unknown
				.clone()
				.unwrap_or_else(FilterItem::default_item),
		}
	}

	pub fn has_locked(&self) -> bool {
		self.known.values().any(|item| item.locked)
			|| self.unknown.as_ref().is_some_and(|item| item.locked)
	}

	pub fn has_any_enabled(&self, possible_values: &[T], has_unknown: bool) -> bool {
		possible_values.iter().any(|value| {
			let item = self.item(Some(value));
			item.enabled || item.locked
		}) || (has_unknown && {
			let item = self.item(None);
			item.enabled || item.locked
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

				if !self.has_any_enabled(possible_values, has_unknown) {
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

#[serializable_struct]
#[derive(Default)]
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

#[serializable_enum]
#[derive(Default)]
pub enum GamesSortBy {
	#[default]
	Title,
	Engine,
	ReleaseDate,
}

#[serializable_struct]
#[derive(Default)]
pub struct GamesQuery {
	pub filter: GamesFilter,
	pub search: String,
	pub sort_by: GamesSortBy,
	pub sort_descending: bool,
}
