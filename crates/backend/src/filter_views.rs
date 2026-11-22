use std::{
	collections::{BTreeMap, HashSet},
	fmt::Display,
	hash::Hash,
	str::FromStr,
};

use strum::IntoEnumIterator;

use crate::{
	architecture::Architecture,
	game_engines::{game_engine::EngineBrand, unity::UnityBackend},
	game_providers::game_provider::GameProviderId,
	game_tag::GameTag,
	games_query::{FilterAction, FilterGroup, FilterGroupKey, GamesFilter, InstallState},
	mods_query::{ModType, ModsFilter},
	operating_system::OperatingSystem,
	result::LogErrExt,
};

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
	ProviderLocalFiles,
	ModTypeInstallable,
	ModTypeLinkOnly,
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
	pub available: bool,
}

pub struct FilterGroupView {
	pub key: FilterGroupKey,
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
			Self::Manual => Some(FilterValueLabel::ProviderLocalFiles),
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

impl FilterValue for ModType {
	const VALUES: &'static [Self] = &Self::ALL;

	fn label(&self) -> Option<FilterValueLabel> {
		Some(match self {
			Self::Installable => FilterValueLabel::ModTypeInstallable,
			Self::LinkOnly => FilterValueLabel::ModTypeLinkOnly,
		})
	}

	fn display_text(&self) -> String {
		match self {
			Self::Installable => "Installable",
			Self::LinkOnly => "Link Only",
		}
		.to_string()
	}
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

fn value_view<T: FilterValue>(
	group: &FilterGroup<T>,
	value: &T,
	available: bool,
) -> FilterValueView {
	let item = group.item(Some(value));

	FilterValueView {
		value: value.to_string(),
		label: value.label(),
		display: value.display_text(),
		note: value.note(),
		enabled: item.enabled,
		locked: item.locked,
		available,
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
		available: true,
	}
}

fn text_view(group: &FilterGroup<String>, value: &str, display: &str) -> FilterValueView {
	let item = group.item(Some(&value.to_string()));

	FilterValueView {
		value: value.to_string(),
		label: None,
		display: display.to_string(),
		note: FilterValueNote::None,
		enabled: item.enabled,
		locked: item.locked,
		available: true,
	}
}

fn known_values_with_availability<T: FilterValue>(
	group: &FilterGroup<T>,
	is_available: impl Fn(&T) -> bool,
) -> Vec<FilterValueView> {
	let mut values: Vec<FilterValueView> = T::VALUES
		.iter()
		.map(|value| value_view(group, value, is_available(value)))
		.collect();
	values.sort_by_key(|value| !value.available);

	if T::HAS_UNKNOWN {
		values.push(unknown_view(group));
	}

	values
}

fn known_values<T: FilterValue>(group: &FilterGroup<T>) -> Vec<FilterValueView> {
	known_values_with_availability(group, |_| true)
}

fn build_group<T: FilterValue>(
	key: FilterGroupKey,
	group: &FilterGroup<T>,
	values: Vec<FilterValueView>,
	collapsed_filters: &HashSet<FilterGroupKey>,
) -> FilterGroupView {
	build_group_view(
		key,
		group,
		values,
		T::VALUES,
		T::HAS_UNKNOWN,
		collapsed_filters,
	)
}

fn build_group_view<T: Hash + Eq>(
	key: FilterGroupKey,
	group: &FilterGroup<T>,
	values: Vec<FilterValueView>,
	possible_values: &[T],
	has_unknown: bool,
	collapsed_filters: &HashSet<FilterGroupKey>,
) -> FilterGroupView {
	FilterGroupView {
		key,
		values,
		has_changes: group.has_changes(possible_values, has_unknown),
		has_locked: group.has_locked(),
		expanded: !collapsed_filters.contains(&key),
	}
}

fn tag_group_view(
	group: &FilterGroup<String>,
	mod_tags: &BTreeMap<String, String>,
	unknown_label: Option<FilterValueLabel>,
	collapsed_filters: &HashSet<FilterGroupKey>,
) -> FilterGroupView {
	let mut values: Vec<FilterValueView> = mod_tags
		.iter()
		.map(|(tag_id, title)| text_view(group, tag_id, title))
		.collect();
	values.sort_by_key(|value| value.display.to_lowercase());

	if let Some(label) = unknown_label {
		let item = group.item(None);
		values.push(FilterValueView {
			value: String::new(),
			label: Some(label),
			display: String::new(),
			note: FilterValueNote::None,
			enabled: item.enabled,
			locked: item.locked,
			available: true,
		});
	}

	let tag_ids = mod_tags.keys().cloned().collect::<Vec<_>>();

	build_group_view(
		FilterGroupKey::ModTags,
		group,
		values,
		&tag_ids,
		unknown_label.is_some(),
		collapsed_filters,
	)
}

fn mod_tag_group(
	filter: &GamesFilter,
	mod_tags: &BTreeMap<String, String>,
	collapsed_filters: &HashSet<FilterGroupKey>,
) -> FilterGroupView {
	let unknown_label = (!mod_tags.is_empty()).then_some(FilterValueLabel::TagUntagged);

	tag_group_view(&filter.mod_tags, mod_tags, unknown_label, collapsed_filters)
}

pub fn build_mods_filter_views(
	filter: &ModsFilter,
	mod_tags: &BTreeMap<String, String>,
	collapsed_filters: &HashSet<FilterGroupKey>,
	has_untagged_mods: bool,
) -> Vec<FilterGroupView> {
	let unknown_label = has_untagged_mods.then_some(FilterValueLabel::TagUntagged);

	vec![
		build_group(
			FilterGroupKey::ModTypes,
			&filter.mod_types,
			known_values(&filter.mod_types),
			collapsed_filters,
		),
		tag_group_view(&filter.mod_tags, mod_tags, unknown_label, collapsed_filters),
	]
}

fn group_view(
	filter: &GamesFilter,
	key: FilterGroupKey,
	mod_tags: &BTreeMap<String, String>,
	collapsed_filters: &HashSet<FilterGroupKey>,
	providers_with_games: &HashSet<GameProviderId>,
) -> Option<FilterGroupView> {
	Some(match key {
		FilterGroupKey::ModTypes => return None,
		FilterGroupKey::Installed => build_group(
			key,
			&filter.installed,
			known_values(&filter.installed),
			collapsed_filters,
		),
		FilterGroupKey::Engines => build_group(
			key,
			&filter.engines,
			known_values(&filter.engines),
			collapsed_filters,
		),
		FilterGroupKey::ModTags => mod_tag_group(filter, mod_tags, collapsed_filters),
		FilterGroupKey::Tags => build_group(
			key,
			&filter.tags,
			known_values(&filter.tags),
			collapsed_filters,
		),
		FilterGroupKey::Providers => build_group(
			key,
			&filter.providers,
			known_values_with_availability(&filter.providers, |provider| {
				providers_with_games.contains(provider)
			}),
			collapsed_filters,
		),
		FilterGroupKey::UnityBackends => build_group(
			key,
			&filter.unity_backends,
			known_values(&filter.unity_backends),
			collapsed_filters,
		),
		FilterGroupKey::ExecutableOs => build_group(
			key,
			&filter.executable_os,
			known_values(&filter.executable_os),
			collapsed_filters,
		),
		FilterGroupKey::SupportedOs => build_group(
			key,
			&filter.supported_os,
			known_values(&filter.supported_os),
			collapsed_filters,
		),
		FilterGroupKey::Architectures => build_group(
			key,
			&filter.architectures,
			known_values(&filter.architectures),
			collapsed_filters,
		),
	})
}

pub fn build_filter_views(
	filter: &GamesFilter,
	mod_tags: &BTreeMap<String, String>,
	collapsed_filters: &HashSet<FilterGroupKey>,
	providers_with_games: &HashSet<GameProviderId>,
) -> Vec<FilterGroupView> {
	FilterGroupKey::iter()
		.filter_map(|key| {
			group_view(
				filter,
				key,
				mod_tags,
				collapsed_filters,
				providers_with_games,
			)
		})
		.collect()
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
	mod_tags: &BTreeMap<String, String>,
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
		FilterGroupKey::ModTags => {
			let tag_ids = mod_tags.keys().cloned().collect::<Vec<_>>();
			apply_string_value(&mut filter.mod_tags, value, &tag_ids, true, action)
		}
		FilterGroupKey::ModTypes => false,
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

pub fn apply_mods_filter_action(
	filter: &mut ModsFilter,
	mod_tags: &BTreeMap<String, String>,
	group_key: FilterGroupKey,
	value: &str,
	action: FilterAction,
) -> bool {
	match group_key {
		FilterGroupKey::ModTags => {
			let tag_ids = mod_tags.keys().cloned().collect::<Vec<_>>();
			apply_string_value(&mut filter.mod_tags, value, &tag_ids, true, action)
		}
		FilterGroupKey::ModTypes => {
			apply_string_value(&mut filter.mod_types, value, &ModType::ALL, false, action)
		}
		_ => false,
	}
}

fn value_key(values: impl Iterator<Item = String>) -> String {
	let mut values: Vec<String> = values.collect();
	values.sort_unstable();
	values.join("\u{1f}")
}

pub fn sync_filter_availability(
	filter: &mut GamesFilter,
	mod_tags: &BTreeMap<String, String>,
	providers_with_games: &HashSet<GameProviderId>,
) -> bool {
	let providers_key = value_key(
		GameProviderId::ALL
			.iter()
			.filter(|provider| providers_with_games.contains(provider))
			.map(ToString::to_string),
	);
	let mod_tags_key = value_key(mod_tags.keys().cloned());

	let providers_changed = filter.providers.sync_availability_key(providers_key);
	let mod_tags_changed = filter.mod_tags.sync_availability_key(mod_tags_key);

	providers_changed || mod_tags_changed
}

pub fn sync_mods_filter_availability(
	filter: &mut ModsFilter,
	mod_tags: &BTreeMap<String, String>,
) -> bool {
	let mod_tags_key = value_key(mod_tags.keys().cloned());
	filter.mod_tags.sync_availability_key(mod_tags_key)
}

pub fn reset_mods_filter_group(filter: &mut ModsFilter, group_key: FilterGroupKey) {
	match group_key {
		FilterGroupKey::ModTags => filter.mod_tags.keep_only_locked(),
		FilterGroupKey::ModTypes => filter.mod_types.keep_only_locked(),
		_ => {}
	}
}

pub fn collapse_mods_filters(collapsed_filters: &mut HashSet<FilterGroupKey>) {
	collapsed_filters.insert(FilterGroupKey::ModTags);
	collapsed_filters.insert(FilterGroupKey::ModTypes);
}

pub fn reset_all_mods_filters(filter: &mut ModsFilter) {
	filter.mod_tags.keep_only_locked();
	filter.mod_types.keep_only_locked();
}

pub fn reset_filter_group(filter: &mut GamesFilter, group_key: FilterGroupKey) {
	match group_key {
		FilterGroupKey::Installed => filter.installed.keep_only_locked(),
		FilterGroupKey::Engines => filter.engines.keep_only_locked(),
		FilterGroupKey::ModTags => filter.mod_tags.keep_only_locked(),
		FilterGroupKey::ModTypes => {}
		FilterGroupKey::Tags => filter.tags.keep_only_locked(),
		FilterGroupKey::Providers => filter.providers.keep_only_locked(),
		FilterGroupKey::UnityBackends => filter.unity_backends.keep_only_locked(),
		FilterGroupKey::ExecutableOs => filter.executable_os.keep_only_locked(),
		FilterGroupKey::SupportedOs => filter.supported_os.keep_only_locked(),
		FilterGroupKey::Architectures => filter.architectures.keep_only_locked(),
	}
}

pub fn collapse_all_filters(collapsed_filters: &mut HashSet<FilterGroupKey>) {
	collapsed_filters.extend(FilterGroupKey::iter());
}

pub fn expand_all_filters(collapsed_filters: &mut HashSet<FilterGroupKey>) {
	collapsed_filters.clear();
}

pub fn reset_all_filters(filter: &mut GamesFilter) {
	for key in FilterGroupKey::iter() {
		reset_filter_group(filter, key);
	}
}
