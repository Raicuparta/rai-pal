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

use crate::{FilterGroupView, FilterValueView};

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
		let Ok(value) = value.parse() else {
			return false;
		};
		Some(value)
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
		FilterGroupKey::Installed => apply_string_value(
			&mut filter.installed,
			value,
			&InstallState::ALL,
			false,
			action,
		),
		FilterGroupKey::Engines => {
			apply_string_value(&mut filter.engines, value, &EngineBrand::ALL, true, action)
		}
		FilterGroupKey::ModFamilies => {
			apply_string_value(&mut filter.mod_families, value, mod_families, false, action)
		}
		FilterGroupKey::Tags => {
			apply_string_value(&mut filter.tags, value, &GameTag::ALL, true, action)
		}
		FilterGroupKey::Providers => apply_string_value(
			&mut filter.providers,
			value,
			&GameProviderId::ALL,
			false,
			action,
		),
		FilterGroupKey::UnityBackends => apply_string_value(
			&mut filter.unity_backends,
			value,
			&UnityBackend::ALL,
			true,
			action,
		),
		FilterGroupKey::ExecutableOs => apply_string_value(
			&mut filter.executable_os,
			value,
			&OperatingSystem::ALL,
			true,
			action,
		),
		FilterGroupKey::SupportedOs => apply_string_value(
			&mut filter.supported_os,
			value,
			&OperatingSystem::ALL,
			true,
			action,
		),
		FilterGroupKey::Architectures => apply_string_value(
			&mut filter.architectures,
			value,
			&Architecture::ALL,
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

fn item_data(
	value: &str,
	label_key: &str,
	label: &str,
	item: FilterItem,
	note_label_key: &str,
) -> FilterValueView {
	FilterValueView {
		value: value.into(),
		label_key: label_key.into(),
		label: label.into(),
		note_label_key: note_label_key.into(),
		enabled: item.enabled,
		locked: item.locked,
	}
}

fn value_note_label_key(value: &str) -> &'static str {
	match value {
		"GameMaker" => "engineGameMakerNotFullySupported",
		"Xbox" => "providerXboxOnlyInstalled",
		_ => "",
	}
}

fn group_data(
	key: FilterGroupKey,
	label_key: &str,
	label: &str,
	values: Vec<FilterValueView>,
	has_changes: bool,
	has_locked: bool,
	expanded_filters: &HashSet<String>,
) -> FilterGroupView {
	FilterGroupView {
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
) -> FilterValueView {
	let value = key.to_string();
	item_data(
		&value,
		label_key,
		label,
		group.item(Some(key)),
		value_note_label_key(&value),
	)
}

fn unknown_value<T: Eq + Hash>(
	group: &FilterGroup<T>,
	label_key: &str,
	label: &str,
) -> FilterValueView {
	item_data("", label_key, label, group.item(None), "")
}

fn build_group<T: Eq + Hash>(
	key: FilterGroupKey,
	label_key: &str,
	label: &str,
	group: &FilterGroup<T>,
	values: Vec<FilterValueView>,
	possible_values: &[T],
	has_unknown: bool,
	expanded_filters: &HashSet<String>,
) -> FilterGroupView {
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
) -> FilterGroupView {
	let mut values: Vec<FilterValueView> = mod_families
		.iter()
		.map(|family| {
			item_data(
				family,
				"",
				family,
				filter.mod_families.item(Some(family)),
				"",
			)
		})
		.collect();
	values.sort_by_key(|a| a.label.to_lowercase());

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

fn installed_values(filter: &GamesFilter) -> Vec<FilterValueView> {
	InstallState::ALL
		.iter()
		.map(|state| {
			let (label_key, label) = match state {
				InstallState::Installed => ("statusInstalled", "Installed"),
				InstallState::NotInstalled => ("statusNotInstalled", "Not installed"),
			};

			value(&filter.installed, state, label_key, label)
		})
		.collect()
}

fn engine_values(filter: &GamesFilter) -> Vec<FilterValueView> {
	EngineBrand::ALL
		.iter()
		.map(|brand| value(&filter.engines, brand, "", &brand.to_string()))
		.chain(std::iter::once(unknown_value(
			&filter.engines,
			"unknown",
			"Unknown",
		)))
		.collect()
}

fn tag_values(filter: &GamesFilter) -> Vec<FilterValueView> {
	GameTag::ALL
		.iter()
		.map(|tag| {
			let (label_key, label) = match tag {
				GameTag::VR => ("tagVr", "Native VR"),
				GameTag::Demo => ("tagDemo", "Demo"),
			};

			value(&filter.tags, tag, label_key, label)
		})
		.chain(std::iter::once(unknown_value(
			&filter.tags,
			"tagUntagged",
			"Untagged",
		)))
		.collect()
}

fn provider_values(filter: &GamesFilter) -> Vec<FilterValueView> {
	GameProviderId::ALL
		.iter()
		.map(|provider| {
			let (label_key, label) = match provider {
				GameProviderId::Amazon => ("", "Amazon"),
				GameProviderId::Epic => ("", "Epic"),
				GameProviderId::Gog => ("", "GOG"),
				GameProviderId::Itch => ("", "itch.io"),
				GameProviderId::Manual => ("providerManual", "Manual"),
				GameProviderId::Steam => ("", "Steam"),
				GameProviderId::Xbox => ("", "Xbox"),
			};

			value(&filter.providers, provider, label_key, label)
		})
		.collect()
}

fn unity_backend_values(filter: &GamesFilter) -> Vec<FilterValueView> {
	UnityBackend::ALL
		.iter()
		.map(|backend| {
			let label = match backend {
				UnityBackend::Il2Cpp => "IL2CPP",
				UnityBackend::Mono => "Mono",
			};

			value(&filter.unity_backends, backend, "", label)
		})
		.chain(std::iter::once(unknown_value(
			&filter.unity_backends,
			"unknown",
			"Unknown",
		)))
		.collect()
}

fn operating_system_values(group: &FilterGroup<OperatingSystem>) -> Vec<FilterValueView> {
	OperatingSystem::ALL
		.iter()
		.map(|os| value(group, os, "", &os.to_string()))
		.chain(std::iter::once(unknown_value(group, "unknown", "Unknown")))
		.collect()
}

fn architecture_values(filter: &GamesFilter) -> Vec<FilterValueView> {
	Architecture::ALL
		.iter()
		.map(|architecture| {
			let (label_key, label) = match architecture {
				Architecture::X64 => ("arch64", "64-bit"),
				Architecture::X86 => ("arch32", "32-bit"),
			};

			value(&filter.architectures, architecture, label_key, label)
		})
		.chain(std::iter::once(unknown_value(
			&filter.architectures,
			"unknown",
			"Unknown",
		)))
		.collect()
}

pub fn build_groups(
	filter: &GamesFilter,
	mod_families: &[String],
	expanded_filters: &HashSet<String>,
) -> Vec<FilterGroupView> {
	vec![
		build_group(
			FilterGroupKey::Installed,
			"status",
			"Install status",
			&filter.installed,
			installed_values(filter),
			&InstallState::ALL,
			false,
			expanded_filters,
		),
		build_group(
			FilterGroupKey::Engines,
			"engine",
			"Engine",
			&filter.engines,
			engine_values(filter),
			&EngineBrand::ALL,
			true,
			expanded_filters,
		),
		build_mod_family_group(filter, mod_families, expanded_filters),
		build_group(
			FilterGroupKey::Tags,
			"tags",
			"Tag",
			&filter.tags,
			tag_values(filter),
			&GameTag::ALL,
			true,
			expanded_filters,
		),
		build_group(
			FilterGroupKey::Providers,
			"provider",
			"Provider",
			&filter.providers,
			provider_values(filter),
			&GameProviderId::ALL,
			false,
			expanded_filters,
		),
		build_group(
			FilterGroupKey::UnityBackends,
			"unityBackend",
			"Unity Backend",
			&filter.unity_backends,
			unity_backend_values(filter),
			&UnityBackend::ALL,
			true,
			expanded_filters,
		),
		build_group(
			FilterGroupKey::ExecutableOs,
			"executableOs",
			"Executable OS",
			&filter.executable_os,
			operating_system_values(&filter.executable_os),
			&OperatingSystem::ALL,
			true,
			expanded_filters,
		),
		build_group(
			FilterGroupKey::SupportedOs,
			"supportedOs",
			"Supported OS",
			&filter.supported_os,
			operating_system_values(&filter.supported_os),
			&OperatingSystem::ALL,
			true,
			expanded_filters,
		),
		build_group(
			FilterGroupKey::Architectures,
			"architecture",
			"Architecture",
			&filter.architectures,
			architecture_values(filter),
			&Architecture::ALL,
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
