use std::collections::{BTreeMap, HashSet};

use crate::{
	game::Game,
	local_database::mod_database::{ModDatabase, ModInfo},
	mods::{manifest::Mod, mod_family::ModFamily},
	result::Result,
};

/// A mod that applies to a game, together with its compatibility and install state.
#[derive(Clone, Debug)]
pub struct GameMod {
	pub manifest: Mod,
	pub info: ModInfo,
	pub has_remote_config: bool,
}

impl GameMod {
	pub const fn installed(&self) -> bool {
		self.info.installed_hash.is_some()
	}
}

#[derive(Clone, Debug)]
pub enum PotentialMod {
	Family(ModFamily),
	Manifest(Box<Mod>),
}

#[derive(Clone, Debug, Default)]
pub struct GameModGroups {
	pub installed: Vec<GameMod>,
	pub available: Vec<GameMod>,
	pub hidden: Vec<GameMod>,
	pub incompatible: Vec<GameMod>,
	pub potential: Vec<PotentialMod>,
}

impl GameModGroups {
	pub fn load<D: ModDatabase>(
		database: &D,
		game: &Game,
		remote_config_ids: &HashSet<String>,
	) -> Result<Self> {
		let mod_infos = database.get_game_mods(&game.provider_id, &game.game_id)?;
		let mods = database.get_mod_map()?;
		let families = database.get_mod_families().unwrap_or_default();

		let mut compatible_parts = Vec::new();
		let mut hidden_parts = Vec::new();
		let mut incompatible_parts = Vec::new();

		for info in mod_infos {
			let Some(manifest) = mods.get(&info.mod_id).cloned() else {
				continue;
			};

			let entry = GameMod {
				has_remote_config: remote_config_ids.contains(&manifest.id),
				manifest,
				info,
			};

			if entry.info.compatible {
				if entry.manifest.hide_from_game_mods_list == Some(true) {
					hidden_parts.push(entry);
				} else {
					compatible_parts.push(entry);
				}
			} else if entry.manifest.hide_from_game_mods_list != Some(true) {
				incompatible_parts.push(entry);
			}
		}

		let compatible_families = compatible_parts
			.iter()
			.chain(hidden_parts.iter())
			.filter_map(|entry| entry.manifest.family.clone())
			.collect::<HashSet<_>>();

		incompatible_parts.retain(|entry| {
			entry
				.manifest
				.family
				.as_ref()
				.is_none_or(|family| !compatible_families.contains(family))
		});

		sort_entries(&mut compatible_parts);
		sort_entries(&mut hidden_parts);
		sort_entries(&mut incompatible_parts);

		let potential = build_potential_mods(&compatible_parts, &families);
		let (installed, available) = compatible_parts.into_iter().partition(GameMod::installed);

		Ok(Self {
			installed,
			available,
			hidden: hidden_parts,
			incompatible: incompatible_parts,
			potential,
		})
	}
}

fn sort_entries(entries: &mut [GameMod]) {
	entries.sort_by(|a, b| {
		let a_date = a.manifest.release_date().unwrap_or_default();
		let b_date = b.manifest.release_date().unwrap_or_default();

		b_date.cmp(a_date).then_with(|| {
			a.manifest
				.title
				.to_lowercase()
				.cmp(&b.manifest.title.to_lowercase())
		})
	});
}

fn build_potential_mods(
	compatible_entries: &[GameMod],
	families: &BTreeMap<String, ModFamily>,
) -> Vec<PotentialMod> {
	let mut family_counts = BTreeMap::<&str, usize>::new();

	for entry in compatible_entries {
		if let Some(family) = &entry.manifest.family {
			*family_counts.entry(family.as_str()).or_default() += 1;
		}
	}

	let mut seen_families = HashSet::new();
	let mut result = Vec::new();

	for entry in compatible_entries {
		let family_id = entry
			.manifest
			.family
			.as_ref()
			.filter(|family| family_counts.get(family.as_str()).copied().unwrap_or(0) > 1);
		let family = family_id.and_then(|family| families.get(family));

		if let (Some(family_id), Some(family)) = (family_id, family) {
			if seen_families.insert(family_id.clone()) {
				result.push(PotentialMod::Family(family.clone()));
			}
		} else {
			result.push(PotentialMod::Manifest(Box::new(entry.manifest.clone())));
		}
	}

	result
}
