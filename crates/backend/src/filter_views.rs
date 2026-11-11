use std::{collections::HashSet, fmt::Display, hash::Hash, str::FromStr};

use crate::{
	architecture::Architecture,
	game_engines::{game_engine::EngineBrand, unity::UnityBackend},
	game_providers::game_provider::GameProviderId,
	game_tag::GameTag,
	games_query::{FilterAction, FilterGroup, FilterGroupKey, GamesFilter, InstallState},
	operating_system::OperatingSystem,
	result::LogErrExt,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FilterGroupLabel {
	Status,
	Engine,
	Mod,
	Tag,
	Provider,
	UnityBackend,
	ExecutableOs,
	SupportedOs,
	Architecture,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FilterValueLabel {
	Unknown,
	Arch64,
	Arch32,
	TagDemo,
	TagVr,
	TagUntagged,
	StatusInstalled,
	StatusNotInstalled,
	ProviderManual,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FilterValueNote {
	#[default]
	None,
	EngineGodotNotFullySupported,
	EngineGameMakerNotFullySupported,
	ProviderXboxOnlyInstalled,
}

pub struct FilterValueView {
	pub value: String,
	pub label: Option<FilterValueLabel>,
	pub display: String,
	pub note: FilterValueNote,
	pub enabled: bool,
	pub locked: bool,
}

pub struct FilterGroupView {
	pub key: FilterGroupKey,
	pub label: FilterGroupLabel,
	pub values: Vec<FilterValueView>,
	pub has_changes: bool,
	pub has_locked: bool,
	pub expanded: bool,
}

trait FilterValue: Eq + Hash + Clone + Display + 'static {
	const VALUES: &'static [Self];
	const HAS_UNKNOWN: bool = false;

	fn label(&self) -> Option<FilterValueLabel> {
		None
	}

	fn display_text(&self) -> String {
		self.to_string()
	}

	fn note(&self) -> FilterValueNote {
		FilterValueNote::None
	}

	fn unknown_label() -> FilterValueLabel {
		FilterValueLabel::Unknown
	}

	fn unknown_display_name() -> &'static str {
		"Unknown"
	}
}

impl FilterValue for InstallState {
	const VALUES: &'static [Self] = &Self::ALL;

	fn label(&self) -> Option<FilterValueLabel> {
		Some(match self {
			Self::Installed => FilterValueLabel::StatusInstalled,
			Self::NotInstalled => FilterValueLabel::StatusNotInstalled,
		})
	}
}

impl FilterValue for EngineBrand {
	const VALUES: &'static [Self] = &Self::ALL;
	const HAS_UNKNOWN: bool = true;

	fn note(&self) -> FilterValueNote {
		match self {
			Self::Godot => FilterValueNote::EngineGodotNotFullySupported,
			Self::GameMaker => FilterValueNote::EngineGameMakerNotFullySupported,
			_ => FilterValueNote::None,
		}
	}
}

impl FilterValue for GameTag {
	const VALUES: &'static [Self] = &Self::ALL;
	const HAS_UNKNOWN: bool = true;

	fn label(&self) -> Option<FilterValueLabel> {
		Some(match self {
			Self::VR => FilterValueLabel::TagVr,
			Self::Demo => FilterValueLabel::TagDemo,
		})
	}

	fn unknown_label() -> FilterValueLabel {
		FilterValueLabel::TagUntagged
	}

	fn unknown_display_name() -> &'static str {
		"Untagged"
	}
}

impl FilterValue for GameProviderId {
	const VALUES: &'static [Self] = &Self::ALL;

	fn label(&self) -> Option<FilterValueLabel> {
		match self {
			Self::Manual => Some(FilterValueLabel::ProviderManual),
			_ => None,
		}
	}

	fn display_text(&self) -> String {
		self.display_name().to_string()
	}

	fn note(&self) -> FilterValueNote {
		match self {
			Self::Xbox => FilterValueNote::ProviderXboxOnlyInstalled,
			_ => FilterValueNote::None,
		}
	}
}

impl FilterValue for UnityBackend {
	const VALUES: &'static [Self] = &Self::ALL;
	const HAS_UNKNOWN: bool = true;

	fn display_text(&self) -> String {
		match self {
			Self::Il2Cpp => "IL2CPP",
			Self::Mono => "Mono",
		}
		.to_string()
	}
}

impl FilterValue for OperatingSystem {
	const VALUES: &'static [Self] = &Self::ALL;
	const HAS_UNKNOWN: bool = true;
}

impl FilterValue for Architecture {
	const VALUES: &'static [Self] = &Self::ALL;
	const HAS_UNKNOWN: bool = true;

	fn label(&self) -> Option<FilterValueLabel> {
		Some(match self {
			Self::X64 => FilterValueLabel::Arch64,
			Self::X86 => FilterValueLabel::Arch32,
		})
	}
}

fn value_view<T: FilterValue>(group: &FilterGroup<T>, value: &T) -> FilterValueView {
	let item = group.item(Some(value));

	FilterValueView {
		value: value.to_string(),
		label: value.label(),
		display: value.display_text(),
		note: value.note(),
		enabled: item.enabled,
		locked: item.locked,
	}
}

fn unknown_view<T: FilterValue>(group: &FilterGroup<T>) -> FilterValueView {
	let item = group.item(None);

	FilterValueView {
		value: String::new(),
		label: Some(T::unknown_label()),
		display: T::unknown_display_name().to_string(),
		note: FilterValueNote::None,
		enabled: item.enabled,
		locked: item.locked,
	}
}

fn text_view(group: &FilterGroup<String>, value: &str) -> FilterValueView {
	let item = group.item(Some(&value.to_string()));

	FilterValueView {
		value: value.to_string(),
		label: None,
		display: value.to_string(),
		note: FilterValueNote::None,
		enabled: item.enabled,
		locked: item.locked,
	}
}

fn known_values<T: FilterValue>(group: &FilterGroup<T>) -> Vec<FilterValueView> {
	let mut values: Vec<FilterValueView> = T::VALUES
		.iter()
		.map(|value| value_view(group, value))
		.collect();

	if T::HAS_UNKNOWN {
		values.push(unknown_view(group));
	}

	values
}

fn build_group<T: FilterValue>(
	key: FilterGroupKey,
	label: FilterGroupLabel,
	group: &FilterGroup<T>,
	values: Vec<FilterValueView>,
	expanded_filters: &HashSet<FilterGroupKey>,
) -> FilterGroupView {
	FilterGroupView {
		key,
		label,
		values,
		has_changes: group.has_changes(T::VALUES, T::HAS_UNKNOWN),
		has_locked: group.has_locked(),
		expanded: expanded_filters.contains(&key),
	}
}

fn mod_family_group(
	filter: &GamesFilter,
	mod_families: &[String],
	expanded_filters: &HashSet<FilterGroupKey>,
) -> FilterGroupView {
	let mut values: Vec<FilterValueView> = mod_families
		.iter()
		.map(|family| text_view(&filter.mod_families, family))
		.collect();
	values.sort_by_key(|value| value.display.to_lowercase());

	FilterGroupView {
		key: FilterGroupKey::ModFamilies,
		label: FilterGroupLabel::Mod,
		values,
		has_changes: filter.mod_families.has_changes(mod_families, false),
		has_locked: filter.mod_families.has_locked(),
		expanded: expanded_filters.contains(&FilterGroupKey::ModFamilies),
	}
}

pub fn build_filter_views(
	filter: &GamesFilter,
	mod_families: &[String],
	expanded_filters: &HashSet<FilterGroupKey>,
) -> Vec<FilterGroupView> {
	vec![
		build_group(
			FilterGroupKey::Installed,
			FilterGroupLabel::Status,
			&filter.installed,
			known_values(&filter.installed),
			expanded_filters,
		),
		build_group(
			FilterGroupKey::Engines,
			FilterGroupLabel::Engine,
			&filter.engines,
			known_values(&filter.engines),
			expanded_filters,
		),
		mod_family_group(filter, mod_families, expanded_filters),
		build_group(
			FilterGroupKey::Tags,
			FilterGroupLabel::Tag,
			&filter.tags,
			known_values(&filter.tags),
			expanded_filters,
		),
		build_group(
			FilterGroupKey::Providers,
			FilterGroupLabel::Provider,
			&filter.providers,
			known_values(&filter.providers),
			expanded_filters,
		),
		build_group(
			FilterGroupKey::UnityBackends,
			FilterGroupLabel::UnityBackend,
			&filter.unity_backends,
			known_values(&filter.unity_backends),
			expanded_filters,
		),
		build_group(
			FilterGroupKey::ExecutableOs,
			FilterGroupLabel::ExecutableOs,
			&filter.executable_os,
			known_values(&filter.executable_os),
			expanded_filters,
		),
		build_group(
			FilterGroupKey::SupportedOs,
			FilterGroupLabel::SupportedOs,
			&filter.supported_os,
			known_values(&filter.supported_os),
			expanded_filters,
		),
		build_group(
			FilterGroupKey::Architectures,
			FilterGroupLabel::Architecture,
			&filter.architectures,
			known_values(&filter.architectures),
			expanded_filters,
		),
	]
}

fn apply_string_value<T>(
	group: &mut FilterGroup<T>,
	value: &str,
	possible_values: &[T],
	has_unknown: bool,
	action: FilterAction,
) -> bool
where
	T: Eq + Hash + Clone + FromStr,
	<T as FromStr>::Err: Display,
{
	let target = if value.is_empty() {
		if !has_unknown {
			return false;
		}
		None
	} else {
		let Some(value) = value.parse().ok_or_log("Failed to parse filter value") else {
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
	group_key: FilterGroupKey,
	value: &str,
	action: FilterAction,
) -> bool {
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

pub fn reset_filter_group(filter: &mut GamesFilter, group_key: FilterGroupKey) {
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
}

pub fn reset_all_filters(filter: &mut GamesFilter) {
	for key in FilterGroupKey::ALL {
		reset_filter_group(filter, key);
	}
}
