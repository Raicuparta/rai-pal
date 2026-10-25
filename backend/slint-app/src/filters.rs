use std::{collections::HashSet, hash::Hash, str::FromStr};

use rai_pal_core::{
	architecture::Architecture,
	game_engines::{game_engine::EngineBrand, unity::UnityBackend},
	game_providers::game_provider::GameProviderId,
	game_tag::GameTag,
	games_query::{
		FilterAction, FilterGroup, FilterGroupKey, FilterItem, GamesFilter, InstallState,
	},
	local_database::{app_database::DbMutex, mod_database::ModDatabase},
	operating_system::OperatingSystem,
};

use crate::{FilterGroupData, FilterValueData};

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

fn apply_string_value<T>(
	group: &mut FilterGroup<T>,
	value: &str,
	possible_values: &[T],
	has_unknown: bool,
	action: FilterAction,
) -> bool
where
	T: Eq + Hash + Clone + FromStr,
{
	let target = if value.is_empty() {
		if !has_unknown {
			return false;
		}
		None
	} else {
		match value.parse() {
			Ok(value) => Some(value),
			Err(_) => return false,
		}
	};

	group.apply_action(target, possible_values, has_unknown, action);
	true
}

pub fn apply_filter_action(
	filter: &mut GamesFilter,
	mod_families: &[String],
	group_key: &str,
	value: &str,
	action: FilterAction,
) -> bool {
	let Ok(group_key) = FilterGroupKey::from_str(group_key) else {
		return false;
	};

	match group_key {
		FilterGroupKey::Installed => {
			apply_string_value(&mut filter.installed, value, INSTALL_STATES, false, action)
		}
		FilterGroupKey::Engines => {
			apply_string_value(&mut filter.engines, value, ENGINES, true, action)
		}
		FilterGroupKey::ModFamilies => {
			apply_string_value(&mut filter.mod_families, value, mod_families, false, action)
		}
		FilterGroupKey::Tags => apply_string_value(&mut filter.tags, value, TAGS, true, action),
		FilterGroupKey::Providers => {
			apply_string_value(&mut filter.providers, value, PROVIDERS, false, action)
		}
		FilterGroupKey::UnityBackends => apply_string_value(
			&mut filter.unity_backends,
			value,
			UNITY_BACKENDS,
			true,
			action,
		),
		FilterGroupKey::ExecutableOs => apply_string_value(
			&mut filter.executable_os,
			value,
			OPERATING_SYSTEMS,
			true,
			action,
		),
		FilterGroupKey::SupportedOs => apply_string_value(
			&mut filter.supported_os,
			value,
			OPERATING_SYSTEMS,
			true,
			action,
		),
		FilterGroupKey::Architectures => apply_string_value(
			&mut filter.architectures,
			value,
			ARCHITECTURES,
			true,
			action,
		),
	}
}

pub fn reset_filter_group(filter: &mut GamesFilter, group_key: &str) -> bool {
	let Ok(group_key) = FilterGroupKey::from_str(group_key) else {
		return false;
	};

	match group_key {
		FilterGroupKey::Installed => filter.installed.keep_only_locked(),
		FilterGroupKey::Engines => filter.engines.keep_only_locked(),
		FilterGroupKey::ModFamilies => filter.mod_families.keep_only_locked(),
		FilterGroupKey::Tags => filter.tags.keep_only_locked(),
		FilterGroupKey::Providers => filter.providers.keep_only_locked(),
		FilterGroupKey::UnityBackends => filter.unity_backends.keep_only_locked(),
		FilterGroupKey::ExecutableOs => filter.executable_os.keep_only_locked(),
		FilterGroupKey::SupportedOs => filter.supported_os.keep_only_locked(),
		FilterGroupKey::Architectures => filter.architectures.keep_only_locked(),
	}

	true
}

pub fn reset_all_filters(filter: &mut GamesFilter) {
	filter.installed.keep_only_locked();
	filter.engines.keep_only_locked();
	filter.mod_families.keep_only_locked();
	filter.tags.keep_only_locked();
	filter.providers.keep_only_locked();
	filter.unity_backends.keep_only_locked();
	filter.executable_os.keep_only_locked();
	filter.supported_os.keep_only_locked();
	filter.architectures.keep_only_locked();
}

fn item_data(value: &str, label_key: &str, label: &str, item: FilterItem) -> FilterValueData {
	FilterValueData {
		value: value.into(),
		label_key: label_key.into(),
		label: label.into(),
		enabled: item.enabled,
		locked: item.locked,
	}
}

fn group_data(
	key: FilterGroupKey,
	label_key: &str,
	label: &str,
	values: Vec<FilterValueData>,
	has_changes: bool,
	has_locked: bool,
	expanded_filters: &HashSet<String>,
) -> FilterGroupData {
	FilterGroupData {
		key: key.as_str().into(),
		label_key: label_key.into(),
		label: label.into(),
		values: values.as_slice().into(),
		has_changes,
		has_locked,
		expanded: expanded_filters.contains(key.as_str()),
	}
}

fn value<T: Eq + Hash + ToString>(
	group: &FilterGroup<T>,
	key: &T,
	label_key: &str,
	label: &str,
) -> FilterValueData {
	item_data(&key.to_string(), label_key, label, group.item(Some(key)))
}

fn unknown_value<T: Eq + Hash>(
	group: &FilterGroup<T>,
	label_key: &str,
	label: &str,
) -> FilterValueData {
	item_data("", label_key, label, group.item(None))
}

fn build_group<T: Eq + Hash>(
	key: FilterGroupKey,
	label_key: &str,
	label: &str,
	group: &FilterGroup<T>,
	values: Vec<FilterValueData>,
	possible_values: &[T],
	has_unknown: bool,
	expanded_filters: &HashSet<String>,
) -> FilterGroupData {
	group_data(
		key,
		label_key,
		label,
		values,
		group.has_changes(possible_values, has_unknown),
		group.has_locked(),
		expanded_filters,
	)
}

fn build_mod_family_group(
	filter: &GamesFilter,
	mod_families: &[String],
	expanded_filters: &HashSet<String>,
) -> FilterGroupData {
	let mut values: Vec<FilterValueData> = mod_families
		.iter()
		.map(|family| item_data(family, "", family, filter.mod_families.item(Some(family))))
		.collect();
	values.sort_by(|a, b| a.label.to_lowercase().cmp(&b.label.to_lowercase()));

	build_group(
		FilterGroupKey::ModFamilies,
		"mod",
		"Mod",
		&filter.mod_families,
		values,
		mod_families,
		false,
		expanded_filters,
	)
}

pub fn build_groups(
	filter: &GamesFilter,
	mod_families: &[String],
	expanded_filters: &HashSet<String>,
) -> Vec<FilterGroupData> {
	let installed_values = vec![
		value(
			&filter.installed,
			&InstallState::Installed,
			"statusInstalled",
			"Installed",
		),
		value(
			&filter.installed,
			&InstallState::NotInstalled,
			"statusNotInstalled",
			"Not installed",
		),
	];

	let engine_values = vec![
		value(&filter.engines, &EngineBrand::Unity, "", "Unity"),
		value(&filter.engines, &EngineBrand::Unreal, "", "Unreal"),
		value(&filter.engines, &EngineBrand::Godot, "", "Godot"),
		value(&filter.engines, &EngineBrand::GameMaker, "", "GameMaker"),
		unknown_value(&filter.engines, "unknown", "Unknown"),
	];

	let tag_values = vec![
		value(&filter.tags, &GameTag::VR, "tagVr", "Native VR"),
		value(&filter.tags, &GameTag::Demo, "tagDemo", "Demo"),
		unknown_value(&filter.tags, "tagUntagged", "Untagged"),
	];

	let provider_values = vec![
		value(&filter.providers, &GameProviderId::Epic, "", "Epic"),
		value(&filter.providers, &GameProviderId::Gog, "", "GOG"),
		value(&filter.providers, &GameProviderId::Itch, "", "itch.io"),
		value(
			&filter.providers,
			&GameProviderId::Manual,
			"providerManual",
			"Manual",
		),
		value(&filter.providers, &GameProviderId::Steam, "", "Steam"),
		value(&filter.providers, &GameProviderId::Xbox, "", "Xbox"),
	];

	let unity_backend_values = vec![
		value(&filter.unity_backends, &UnityBackend::Il2Cpp, "", "IL2CPP"),
		value(&filter.unity_backends, &UnityBackend::Mono, "", "Mono"),
		unknown_value(&filter.unity_backends, "unknown", "Unknown"),
	];

	let executable_os_values = vec![
		value(
			&filter.executable_os,
			&OperatingSystem::Windows,
			"",
			"Windows",
		),
		value(&filter.executable_os, &OperatingSystem::Linux, "", "Linux"),
		unknown_value(&filter.executable_os, "unknown", "Unknown"),
	];

	let supported_os_values = vec![
		value(
			&filter.supported_os,
			&OperatingSystem::Windows,
			"",
			"Windows",
		),
		value(&filter.supported_os, &OperatingSystem::Linux, "", "Linux"),
		unknown_value(&filter.supported_os, "unknown", "Unknown"),
	];

	let architecture_values = vec![
		value(
			&filter.architectures,
			&Architecture::X64,
			"arch64",
			"64-bit",
		),
		value(
			&filter.architectures,
			&Architecture::X86,
			"arch32",
			"32-bit",
		),
		unknown_value(&filter.architectures, "unknown", "Unknown"),
	];

	vec![
		build_group(
			FilterGroupKey::Installed,
			"status",
			"Install status",
			&filter.installed,
			installed_values,
			INSTALL_STATES,
			false,
			expanded_filters,
		),
		build_group(
			FilterGroupKey::Engines,
			"engine",
			"Engine",
			&filter.engines,
			engine_values,
			ENGINES,
			true,
			expanded_filters,
		),
		build_mod_family_group(filter, mod_families, expanded_filters),
		build_group(
			FilterGroupKey::Tags,
			"tags",
			"Tag",
			&filter.tags,
			tag_values,
			TAGS,
			true,
			expanded_filters,
		),
		build_group(
			FilterGroupKey::Providers,
			"provider",
			"Provider",
			&filter.providers,
			provider_values,
			PROVIDERS,
			false,
			expanded_filters,
		),
		build_group(
			FilterGroupKey::UnityBackends,
			"unityBackend",
			"Unity Backend",
			&filter.unity_backends,
			unity_backend_values,
			UNITY_BACKENDS,
			true,
			expanded_filters,
		),
		build_group(
			FilterGroupKey::ExecutableOs,
			"executableOs",
			"Executable OS",
			&filter.executable_os,
			executable_os_values,
			OPERATING_SYSTEMS,
			true,
			expanded_filters,
		),
		build_group(
			FilterGroupKey::SupportedOs,
			"supportedOs",
			"Supported OS",
			&filter.supported_os,
			supported_os_values,
			OPERATING_SYSTEMS,
			true,
			expanded_filters,
		),
		build_group(
			FilterGroupKey::Architectures,
			"architecture",
			"Architecture",
			&filter.architectures,
			architecture_values,
			ARCHITECTURES,
			true,
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
