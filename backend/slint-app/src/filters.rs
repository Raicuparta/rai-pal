use std::hash::Hash;

use rai_pal_core::{
	architecture::Architecture,
	game_engines::{game_engine::EngineBrand, unity::UnityBackend},
	game_providers::game_provider::GameProviderId,
	game_tag::GameTag,
	games_query::{FilterGroup, FilterItem, GamesFilter, InstallState},
	local_database::{app_database::DbMutex, mod_database::ModDatabase},
	operating_system::OperatingSystem,
};

use crate::{FilterGroupData, FilterValueData};

#[derive(Clone, Copy)]
pub enum FilterAction {
	Toggle(bool),
	Lock,
	Only,
	Reset,
}

fn default_item() -> FilterItem {
	FilterItem {
		enabled: true,
		locked: false,
	}
}

fn get_item<T: Eq + Hash>(group: &FilterGroup<T>, key: Option<&T>) -> FilterItem {
	match key {
		Some(key) => group.known.get(key).cloned().unwrap_or_else(default_item),
		None => group.unknown.clone().unwrap_or_else(default_item),
	}
}

fn set_item<T: Eq + Hash + Clone>(group: &mut FilterGroup<T>, key: Option<T>, item: FilterItem) {
	if key.is_none() && group.unknown.is_none() && item.enabled && !item.locked {
		return;
	}

	if item.enabled && !item.locked {
		if let Some(key) = key {
			group.known.remove(&key);
		} else {
			group.unknown = None;
		}
	} else if let Some(key) = key {
		group.known.insert(key, item);
	} else {
		group.unknown = Some(item);
	}
}

fn keep_only_locked<T: Eq + Hash>(group: &mut FilterGroup<T>) {
	group.known.retain(|_, item| item.locked);

	if group.unknown.as_ref().is_some_and(|item| !item.locked) {
		group.unknown = None;
	}
}

fn has_any_enabled<T: Eq + Hash>(
	group: &FilterGroup<T>,
	possible_values: &[T],
	has_unknown: bool,
) -> bool {
	let known_enabled = possible_values.iter().any(|value| {
		let item = get_item(group, Some(value));
		item.enabled || item.locked
	});

	known_enabled
		|| (has_unknown && {
			let item = get_item(group, None);
			item.enabled || item.locked
		})
}

fn has_changes<T: Eq + Hash>(
	group: &FilterGroup<T>,
	possible_values: &[T],
	has_unknown: bool,
) -> bool {
	let known_changed = possible_values.iter().any(|value| {
		let item = get_item(group, Some(value));
		!item.enabled && !item.locked
	});

	known_changed
		|| (has_unknown && {
			let item = get_item(group, None);
			!item.enabled && !item.locked
		})
}

fn has_locked<T: Eq + Hash>(group: &FilterGroup<T>) -> bool {
	group.known.values().any(|item| item.locked)
		|| group.unknown.as_ref().is_some_and(|item| item.locked)
}

fn apply_to_group<T: Eq + Hash + Clone>(
	group: &mut FilterGroup<T>,
	target: Option<T>,
	possible_values: &[T],
	has_unknown: bool,
	action: FilterAction,
) {
	match action {
		FilterAction::Toggle(enabled) => {
			let mut item = get_item(group, target.as_ref());
			item.enabled = enabled;
			set_item(group, target, item);

			if !has_any_enabled(group, possible_values, has_unknown) {
				keep_only_locked(group);
			}
		}
		FilterAction::Lock => {
			let mut item = get_item(group, target.as_ref());
			item.locked = !item.locked;
			set_item(group, target, item);
		}
		FilterAction::Only => {
			for value in possible_values {
				if target.as_ref() == Some(value) {
					let mut item = get_item(group, Some(value));
					item.enabled = true;
					set_item(group, Some(value.clone()), item);
				} else {
					let item = get_item(group, Some(value));
					if !item.locked {
						set_item(
							group,
							Some(value.clone()),
							FilterItem {
								enabled: false,
								locked: false,
							},
						);
					}
				}
			}

			if has_unknown {
				if target.is_none() {
					let mut item = get_item(group, None);
					item.enabled = true;
					set_item(group, None, item);
				} else {
					let item = get_item(group, None);
					if !item.locked {
						set_item(
							group,
							None,
							FilterItem {
								enabled: false,
								locked: false,
							},
						);
					}
				}
			}
		}
		FilterAction::Reset => keep_only_locked(group),
	}
}

const PROVIDERS: &[GameProviderId] = &[
	GameProviderId::Epic,
	GameProviderId::Gog,
	GameProviderId::Itch,
	GameProviderId::Manual,
	GameProviderId::Steam,
	GameProviderId::Xbox,
];

const ENGINES: &[EngineBrand] = &[
	EngineBrand::Unity,
	EngineBrand::Unreal,
	EngineBrand::Godot,
	EngineBrand::GameMaker,
];

const TAGS: &[GameTag] = &[GameTag::VR, GameTag::Demo];

const UNITY_BACKENDS: &[UnityBackend] = &[UnityBackend::Il2Cpp, UnityBackend::Mono];

const OPERATING_SYSTEMS: &[OperatingSystem] = &[OperatingSystem::Windows, OperatingSystem::Linux];

const ARCHITECTURES: &[Architecture] = &[Architecture::X64, Architecture::X86];

const INSTALL_STATES: &[InstallState] = &[InstallState::Installed, InstallState::NotInstalled];

macro_rules! run_on_group {
	($filter:expr, $field:ident, $ty:ty, $possible:expr, $has_unknown:expr, $value:expr, $action:expr) => {{
		let possible_values: Vec<$ty> = $possible.to_vec();
		let value_text: &str = $value;
		let target = if value_text.is_empty() && $has_unknown {
			None
		} else {
			value_text.parse::<$ty>().ok()
		};

		if value_text.is_empty() && !$has_unknown {
			false
		} else if target.is_some() || $has_unknown {
			apply_to_group(
				&mut $filter.$field,
				target,
				&possible_values,
				$has_unknown,
				$action,
			);
			true
		} else {
			false
		}
	}};
}

pub fn apply_filter_action(
	filter: &mut GamesFilter,
	mod_families: &[String],
	group_key: &str,
	value: &str,
	action: FilterAction,
) -> bool {
	match group_key {
		"installed" => {
			run_on_group!(
				filter,
				installed,
				InstallState,
				INSTALL_STATES,
				false,
				value,
				action
			)
		}
		"engines" => run_on_group!(filter, engines, EngineBrand, ENGINES, true, value, action),
		"modFamilies" => {
			run_on_group!(
				filter,
				mod_families,
				String,
				mod_families,
				false,
				value,
				action
			)
		}
		"tags" => run_on_group!(filter, tags, GameTag, TAGS, true, value, action),
		"providers" => {
			run_on_group!(
				filter,
				providers,
				GameProviderId,
				PROVIDERS,
				false,
				value,
				action
			)
		}
		"unityBackends" => run_on_group!(
			filter,
			unity_backends,
			UnityBackend,
			UNITY_BACKENDS,
			true,
			value,
			action
		),
		"executableOs" => run_on_group!(
			filter,
			executable_os,
			OperatingSystem,
			OPERATING_SYSTEMS,
			true,
			value,
			action
		),
		"supportedOs" => run_on_group!(
			filter,
			supported_os,
			OperatingSystem,
			OPERATING_SYSTEMS,
			true,
			value,
			action
		),
		"architectures" => run_on_group!(
			filter,
			architectures,
			Architecture,
			ARCHITECTURES,
			true,
			value,
			action
		),
		_ => false,
	}
}

pub fn reset_all_filters(filter: &mut GamesFilter) {
	keep_only_locked(&mut filter.installed);
	keep_only_locked(&mut filter.engines);
	keep_only_locked(&mut filter.mod_families);
	keep_only_locked(&mut filter.tags);
	keep_only_locked(&mut filter.providers);
	keep_only_locked(&mut filter.unity_backends);
	keep_only_locked(&mut filter.executable_os);
	keep_only_locked(&mut filter.supported_os);
	keep_only_locked(&mut filter.architectures);
}

fn item_data(value: &str, label: &str, item: FilterItem) -> FilterValueData {
	FilterValueData {
		value: value.into(),
		label: label.into(),
		enabled: item.enabled,
		locked: item.locked,
	}
}

fn group_data(
	key: &str,
	label: &str,
	values: Vec<FilterValueData>,
	has_changes: bool,
	has_locked: bool,
	expanded_filters: &std::collections::HashSet<String>,
) -> FilterGroupData {
	FilterGroupData {
		key: key.into(),
		label: label.into(),
		values: values.as_slice().into(),
		has_changes,
		has_locked,
		expanded: expanded_filters.contains(key),
	}
}

fn value<T: Eq + Hash + ToString>(
	group: &FilterGroup<T>,
	raw_value: &str,
	key: &T,
	label: &str,
) -> FilterValueData {
	let _ = raw_value;
	item_data(&key.to_string(), label, get_item(group, Some(key)))
}

fn unknown_value<T: Eq + Hash>(group: &FilterGroup<T>, label: &str) -> FilterValueData {
	item_data("", label, get_item(group, None))
}

pub fn build_groups(
	filter: &GamesFilter,
	mod_families: &[String],
	expanded_filters: &std::collections::HashSet<String>,
) -> Vec<FilterGroupData> {
	let installed_values = vec![
		value(
			&filter.installed,
			"Installed",
			&InstallState::Installed,
			"Installed",
		),
		value(
			&filter.installed,
			"NotInstalled",
			&InstallState::NotInstalled,
			"Not installed",
		),
	];

	let engine_values = vec![
		value(&filter.engines, "Unity", &EngineBrand::Unity, "Unity"),
		value(&filter.engines, "Unreal", &EngineBrand::Unreal, "Unreal"),
		value(&filter.engines, "Godot", &EngineBrand::Godot, "Godot"),
		value(
			&filter.engines,
			"GameMaker",
			&EngineBrand::GameMaker,
			"GameMaker",
		),
		unknown_value(&filter.engines, "Unknown"),
	];

	let mut family_values: Vec<FilterValueData> = mod_families
		.iter()
		.map(|family| item_data(family, family, get_item(&filter.mod_families, Some(family))))
		.collect();
	family_values.sort_by(|a, b| a.label.to_lowercase().cmp(&b.label.to_lowercase()));

	let tag_values = vec![
		value(&filter.tags, "VR", &GameTag::VR, "Native VR"),
		value(&filter.tags, "Demo", &GameTag::Demo, "Demo"),
		unknown_value(&filter.tags, "Untagged"),
	];

	let provider_values = vec![
		value(&filter.providers, "Epic", &GameProviderId::Epic, "Epic"),
		value(&filter.providers, "Gog", &GameProviderId::Gog, "GOG"),
		value(&filter.providers, "Itch", &GameProviderId::Itch, "itch.io"),
		value(
			&filter.providers,
			"Manual",
			&GameProviderId::Manual,
			"Manual",
		),
		value(&filter.providers, "Steam", &GameProviderId::Steam, "Steam"),
		value(&filter.providers, "Xbox", &GameProviderId::Xbox, "Xbox"),
	];

	let unity_backend_values = vec![
		value(
			&filter.unity_backends,
			"Il2Cpp",
			&UnityBackend::Il2Cpp,
			"IL2CPP",
		),
		value(&filter.unity_backends, "Mono", &UnityBackend::Mono, "Mono"),
		unknown_value(&filter.unity_backends, "Unknown"),
	];

	let executable_os_values = vec![
		value(
			&filter.executable_os,
			"Windows",
			&OperatingSystem::Windows,
			"Windows",
		),
		value(
			&filter.executable_os,
			"Linux",
			&OperatingSystem::Linux,
			"Linux",
		),
		unknown_value(&filter.executable_os, "Unknown"),
	];

	let supported_os_values = vec![
		value(
			&filter.supported_os,
			"Windows",
			&OperatingSystem::Windows,
			"Windows",
		),
		value(
			&filter.supported_os,
			"Linux",
			&OperatingSystem::Linux,
			"Linux",
		),
		unknown_value(&filter.supported_os, "Unknown"),
	];

	let architecture_values = vec![
		value(&filter.architectures, "X64", &Architecture::X64, "64-bit"),
		value(&filter.architectures, "X86", &Architecture::X86, "32-bit"),
		unknown_value(&filter.architectures, "Unknown"),
	];

	vec![
		group_data(
			"installed",
			"Install status",
			installed_values,
			has_changes(&filter.installed, INSTALL_STATES, false),
			has_locked(&filter.installed),
			expanded_filters,
		),
		group_data(
			"engines",
			"Engine",
			engine_values,
			has_changes(&filter.engines, ENGINES, true),
			has_locked(&filter.engines),
			expanded_filters,
		),
		group_data(
			"modFamilies",
			"Mod",
			family_values,
			has_changes(&filter.mod_families, mod_families, false),
			has_locked(&filter.mod_families),
			expanded_filters,
		),
		group_data(
			"tags",
			"Tag",
			tag_values,
			has_changes(&filter.tags, TAGS, true),
			has_locked(&filter.tags),
			expanded_filters,
		),
		group_data(
			"providers",
			"Provider",
			provider_values,
			has_changes(&filter.providers, PROVIDERS, false),
			has_locked(&filter.providers),
			expanded_filters,
		),
		group_data(
			"unityBackends",
			"Unity Backend",
			unity_backend_values,
			has_changes(&filter.unity_backends, UNITY_BACKENDS, true),
			has_locked(&filter.unity_backends),
			expanded_filters,
		),
		group_data(
			"executableOs",
			"Executable OS",
			executable_os_values,
			has_changes(&filter.executable_os, OPERATING_SYSTEMS, true),
			has_locked(&filter.executable_os),
			expanded_filters,
		),
		group_data(
			"supportedOs",
			"Supported OS",
			supported_os_values,
			has_changes(&filter.supported_os, OPERATING_SYSTEMS, true),
			has_locked(&filter.supported_os),
			expanded_filters,
		),
		group_data(
			"architectures",
			"Architecture",
			architecture_values,
			has_changes(&filter.architectures, ARCHITECTURES, true),
			has_locked(&filter.architectures),
			expanded_filters,
		),
	]
}

pub fn load_mod_families(database: &DbMutex) -> Vec<String> {
	let Ok(mods) = database.get_mod_map() else {
		return Vec::new();
	};

	let mut families: Vec<String> = mods
		.values()
		.filter_map(|game_mod| game_mod.family.clone())
		.collect();
	families.sort_by_key(|family| family.to_lowercase());
	families.dedup();
	families
}
