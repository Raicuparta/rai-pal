use std::collections::HashSet;

use crate::{
	game::Game,
	local_database::mod_database::{ModDatabase, ModInfo},
	mods::manifest::Mod,
	result::Result,
};

#[derive(Clone, Debug)]
pub struct GameMod {
	pub manifest: Mod,
	pub info: ModInfo,
}

impl GameMod {
	pub const fn installed(&self) -> bool {
		self.info.is_installed
	}

	pub const fn game_specific(&self) -> bool {
		self.manifest.game_ids.is_some()
	}
}

#[derive(Clone, Debug, Default)]
pub struct GameModGroups {
	pub installed: Vec<GameMod>,
	pub game_specific: Vec<GameMod>,
	pub available: Vec<GameMod>,
	pub dependency_only: Vec<GameMod>,
	pub incompatible: Vec<GameMod>,
}

impl GameModGroups {
	pub fn load<D: ModDatabase>(database: &D, game: &Game) -> Result<Self> {
		let mod_infos = database.get_game_mods(&game.provider_id, &game.game_id)?;
		let mods = database.get_mod_map()?;

		let mut compatible_parts = Vec::new();
		let mut dependency_only_parts = Vec::new();
		let mut incompatible_parts = Vec::new();

		for file_name in database.get_broken_installed_manifests(game)? {
			compatible_parts.push(GameMod {
				manifest: broken_manifest(file_name.clone()),
				info: ModInfo {
					mod_id: file_name,
					installed_version: None,
					is_installed: true,
					is_in_catalog: false,
					broken: true,
					is_outdated: false,
					has_installed_dependants: false,
					compatible: true,
					tags: Vec::new(),
				},
			});
		}

		for info in mod_infos {
			let Some(manifest) = mods.get(&info.mod_id).cloned() else {
				continue;
			};

			let entry = GameMod { manifest, info };

			let dependency_only = entry
				.manifest
				.install
				.as_ref()
				.is_some_and(|install| install.dependency_only);

			if entry.info.compatible {
				if dependency_only {
					dependency_only_parts.push(entry);
				} else {
					compatible_parts.push(entry);
				}
			} else if !dependency_only {
				incompatible_parts.push(entry);
			}
		}

		let compatible_tags = compatible_parts
			.iter()
			.chain(dependency_only_parts.iter())
			.filter_map(|entry| entry.manifest.tags.as_ref())
			.flatten()
			.cloned()
			.collect::<HashSet<_>>();

		incompatible_parts.retain(|entry| {
			let tags = entry.manifest.tags.as_deref().unwrap_or_default();

			tags.is_empty() || !tags.iter().all(|tag| compatible_tags.contains(tag))
		});

		sort_entries(&mut compatible_parts);
		sort_entries(&mut dependency_only_parts);
		sort_entries(&mut incompatible_parts);

		let (installed, available): (Vec<GameMod>, Vec<GameMod>) =
			compatible_parts.into_iter().partition(GameMod::installed);
		let (game_specific, available): (Vec<GameMod>, Vec<GameMod>) =
			available.into_iter().partition(GameMod::game_specific);

		Ok(Self {
			installed,
			game_specific,
			available,
			dependency_only: dependency_only_parts,
			incompatible: incompatible_parts,
		})
	}
}

fn broken_manifest(file_name: String) -> Mod {
	Mod {
		id: file_name.clone(),
		tags: None,
		game_ids: None,
		title: file_name,
		author: String::new(),
		website: None,
		description: String::new(),
		download: None,
		engine: None,
		engine_version_range: None,
		unity_backend: None,
		architecture: None,
		game_os: None,
		host_os: None,
		deprecated: Some(true),
		paths: None,
		optional_dependencies: None,
		required_dependencies: None,
		install: None,
		run_for_game: None,
		run_standalone: None,
		game_environment: None,
		hash: None,
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
