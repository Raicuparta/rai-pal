use std::{
	collections::{BTreeMap, HashSet},
	rc::Rc,
	sync::Arc,
	thread,
};

use rai_pal_core::{
	game::DbGame,
	game_engines::game_engine::{EngineVersionNumbers, EngineVersionRange},
	local_database::{
		app_database::DbMutex,
		mod_database::{GameModInfo, ModDatabase},
	},
	mods::{
		game_mod::{GameMod, ModDependency, install_steps},
		mod_family::ModFamily,
	},
	result::Result,
};
use slint::ComponentHandle;

use crate::{GameModRowData, MainWindow, PotentialModRowData, downloads::Tracker, notifications};

#[derive(Default)]
struct LoadedMods {
	installed: Vec<GameModRowData>,
	available: Vec<GameModRowData>,
	hidden: Vec<GameModRowData>,
	incompatible: Vec<GameModRowData>,
	potential: Vec<PotentialModRowData>,
	has_any_mods: bool,
}

pub fn refresh_mods(window: &MainWindow, database: &Arc<DbMutex>, game: &DbGame) {
	apply_mods(window, database, game, &HashSet::new());
	refresh_remote_configs(window.as_weak(), database.clone(), game.clone());
}

fn refresh_remote_configs(window: slint::Weak<MainWindow>, database: Arc<DbMutex>, game: DbGame) {
	if game.exe_path.is_none() {
		return;
	}

	thread::spawn(move || {
		let Ok(runtime) = tokio::runtime::Builder::new_current_thread()
			.enable_all()
			.build()
		else {
			return;
		};

		let Ok(Some(configs)) = runtime.block_on(game.get_remote_configs()) else {
			return;
		};

		let Ok(mods) = database.get_mod_map() else {
			return;
		};

		let remote_config_ids = mods
			.into_values()
			.filter(|game_mod| configs.get_for_mod(game_mod).is_some())
			.map(|game_mod| game_mod.id)
			.collect::<HashSet<_>>();

		let database = database.clone();
		let game = game.clone();

		let _ = slint::invoke_from_event_loop(move || {
			let Some(window) = window.upgrade() else {
				return;
			};

			if !window.get_game_selected()
				|| window.get_selected_game_row().game_id.to_string() != game.game_id
			{
				return;
			}

			apply_mods(&window, &database, &game, &remote_config_ids);
		});
	});
}

fn apply_mods(
	window: &MainWindow,
	database: &DbMutex,
	game: &DbGame,
	remote_config_ids: &HashSet<String>,
) {
	let loaded = load_mods(database, game, remote_config_ids);

	window.set_has_any_mods(loaded.has_any_mods);
	window.set_has_installed_mods(!loaded.installed.is_empty());
	window.set_has_available_mods(!loaded.available.is_empty());
	window.set_has_hidden_mods(!loaded.hidden.is_empty());
	window.set_has_incompatible_mods(!loaded.incompatible.is_empty());
	window.set_game_mods_installed(Rc::new(slint::VecModel::from(loaded.installed)).into());
	window.set_game_mods_available(Rc::new(slint::VecModel::from(loaded.available)).into());
	window.set_game_mods_hidden(Rc::new(slint::VecModel::from(loaded.hidden)).into());
	window.set_game_mods_incompatible(Rc::new(slint::VecModel::from(loaded.incompatible)).into());
	window.set_potential_mods(Rc::new(slint::VecModel::from(loaded.potential)).into());
}

pub fn install_mod(
	database: Arc<DbMutex>,
	window: slint::Weak<MainWindow>,
	game: DbGame,
	mod_id: String,
	downloads: Tracker,
) {
	thread::spawn(move || {
		let Ok(runtime) = tokio::runtime::Builder::new_current_thread()
			.enable_all()
			.build()
		else {
			return;
		};

		let (game_mod, deps_to_install) = match collect_install_mods(&database, &game, &mod_id) {
			Ok(mods) => mods,
			Err(err) => {
				eprintln!("Failed to prepare mod install {mod_id}: {err}");
				notifications::error_async(
					&window,
					format!("Failed to install mod {mod_id}: {err}"),
				);
				return;
			}
		};

		let all_owned = deps_to_install
			.iter()
			.chain(std::iter::once(&game_mod))
			.cloned()
			.collect::<Vec<_>>();
		let steps = install_steps(&all_owned);
		downloads.pending_steps(&steps);

		let result = runtime.block_on(install_mod_inner(
			&database,
			&game,
			game_mod,
			deps_to_install,
			&downloads,
		));

		if let Err(err) = result {
			eprintln!("Failed to install mod {mod_id}: {err}");
			downloads.fail_steps(&steps);
			notifications::error_async(&window, format!("Failed to install mod {mod_id}: {err}"));
			return;
		}

		downloads.finish_steps(&steps);
		refresh_after_action(&window, &database, &game);
	});
}

pub fn download_mod_config(
	database: Arc<DbMutex>,
	window: slint::Weak<MainWindow>,
	game: DbGame,
	mod_id: String,
) {
	thread::spawn(move || {
		let Ok(runtime) = tokio::runtime::Builder::new_current_thread()
			.enable_all()
			.build()
		else {
			return;
		};

		let result = runtime.block_on(async {
			database
				.get_mod(&mod_id)?
				.download_remote_config(&game, true)
				.await?;
			database.refresh_installed_mods()?;
			Ok::<(), rai_pal_core::result::Error>(())
		});

		if let Err(err) = result {
			eprintln!("Failed to download config for mod {mod_id}: {err}");
			notifications::error_async(
				&window,
				format!("Failed to download config for mod {mod_id}: {err}"),
			);
			return;
		}

		refresh_after_action(&window, &database, &game);
	});
}

pub fn uninstall_mod(
	database: Arc<DbMutex>,
	window: slint::Weak<MainWindow>,
	game: DbGame,
	mod_id: String,
) {
	thread::spawn(move || {
		let Ok(runtime) = tokio::runtime::Builder::new_current_thread()
			.enable_all()
			.build()
		else {
			return;
		};

		let result = runtime.block_on(async {
			database
				.try_get_installed_mod(&game.provider_id, &game.game_id, &mod_id)?
				.uninstall()
				.await?;
			database.refresh_installed_mods()?;
			Ok::<(), rai_pal_core::result::Error>(())
		});

		if let Err(err) = result {
			eprintln!("Failed to uninstall mod {mod_id}: {err}");
			notifications::error_async(&window, format!("Failed to uninstall mod {mod_id}: {err}"));
			return;
		}

		refresh_after_action(&window, &database, &game);
	});
}

pub fn uninstall_all_mods(database: Arc<DbMutex>, window: slint::Weak<MainWindow>, game: DbGame) {
	thread::spawn(move || {
		let Ok(runtime) = tokio::runtime::Builder::new_current_thread()
			.enable_all()
			.build()
		else {
			return;
		};

		let result = runtime.block_on(async {
			for installed_mod in database.get_installed_mods_for_game(&game)? {
				installed_mod.uninstall().await?;
			}

			game.delete_mods_folder().await?;
			database.refresh_installed_mods()?;
			Ok::<(), rai_pal_core::result::Error>(())
		});

		if let Err(err) = result {
			eprintln!("Failed to uninstall all mods: {err}");
			notifications::error_async(&window, format!("Failed to uninstall all mods: {err}"));
			return;
		}

		refresh_after_action(&window, &database, &game);
	});
}

pub fn run_mod(database: Arc<DbMutex>, game: DbGame, mod_id: String) {
	thread::spawn(move || {
		let result = database
			.get_mod(&mod_id)
			.and_then(|game_mod| game_mod.run(Some(&game)));

		if let Err(err) = result {
			eprintln!("Failed to run mod {mod_id}: {err}");
		}
	});
}

pub fn open_mod_folder(database: Arc<DbMutex>, game: DbGame, mod_id: String) {
	thread::spawn(move || {
		let result = database
			.try_get_installed_mod(&game.provider_id, &game.game_id, &mod_id)
			.and_then(|installed_mod| installed_mod.open_folder());

		if let Err(err) = result {
			eprintln!("Failed to open mod folder {mod_id}: {err}");
		}
	});
}

pub fn configure_mod(database: Arc<DbMutex>, game: DbGame, mod_id: String, open_folder: bool) {
	thread::spawn(move || {
		let result = database
			.try_get_installed_mod(&game.provider_id, &game.game_id, &mod_id)
			.and_then(|installed_mod| installed_mod.configure(open_folder));

		if let Err(err) = result {
			eprintln!("Failed to configure mod {mod_id}: {err}");
		}
	});
}

fn load_mods(database: &DbMutex, game: &DbGame, remote_config_ids: &HashSet<String>) -> LoadedMods {
	let Ok(mod_infos) = database.get_game_mods(&game.provider_id, &game.game_id) else {
		return LoadedMods::default();
	};
	let Ok(mods) = database.get_mod_map() else {
		return LoadedMods::default();
	};
	let families = database.get_mod_families().unwrap_or_default();

	let mut compatible_parts = Vec::new();
	let mut hidden_parts = Vec::new();
	let mut incompatible_parts = Vec::new();

	for info in &mod_infos {
		let Some(game_mod) = mods.get(&info.mod_id) else {
			continue;
		};

		if info.compatible {
			if game_mod.hide_from_game_mods_list == Some(true) {
				hidden_parts.push((game_mod, info));
			} else {
				compatible_parts.push((game_mod, info));
			}
		} else if game_mod.hide_from_game_mods_list != Some(true) {
			incompatible_parts.push((game_mod, info));
		}
	}

	let compatible_families = compatible_parts
		.iter()
		.chain(hidden_parts.iter())
		.filter_map(|(game_mod, _)| game_mod.family.clone())
		.collect::<HashSet<_>>();

	incompatible_parts.retain(|(game_mod, _)| {
		game_mod
			.family
			.as_ref()
			.is_none_or(|family| !compatible_families.contains(family))
	});

	sort_parts(&mut compatible_parts);
	sort_parts(&mut hidden_parts);
	sort_parts(&mut incompatible_parts);

	let compatible_rows = compatible_parts
		.into_iter()
		.map(|(game_mod, info)| make_row(game_mod, info, game, remote_config_ids))
		.collect::<Vec<_>>();
	let hidden = hidden_parts
		.into_iter()
		.map(|(game_mod, info)| make_row(game_mod, info, game, remote_config_ids))
		.collect::<Vec<_>>();
	let incompatible = incompatible_parts
		.into_iter()
		.map(|(game_mod, info)| make_row(game_mod, info, game, remote_config_ids))
		.collect::<Vec<_>>();

	let has_any_mods = !compatible_rows.is_empty() || !incompatible.is_empty();
	let potential = build_potential_rows(&compatible_rows, &families);
	let (installed, available) = compatible_rows.into_iter().partition(|row| row.installed);

	LoadedMods {
		installed,
		available,
		hidden,
		incompatible,
		potential,
		has_any_mods,
	}
}

fn make_row(
	game_mod: &GameMod,
	info: &GameModInfo,
	game: &DbGame,
	remote_config_ids: &HashSet<String>,
) -> GameModRowData {
	let installed = info.installed_hash.is_some();
	let is_incompatible = !info.compatible;
	let can_show_actions = !is_incompatible && game.exe_path.is_some();
	let has_remote_config = remote_config_ids.contains(&game_mod.id);

	GameModRowData {
		mod_id: game_mod.id.clone().into(),
		title: game_mod.title.clone().into(),
		description: game_mod.description.clone().into(),
		family: game_mod.family.clone().unwrap_or_default().into(),
		version_text: version_text(info, game_mod).into(),
		release_date: release_date(game_mod).into(),
		author: game_mod.author.clone().into(),
		engine_text: engine_text(game_mod).into(),
		installed,
		outdated: info.is_outdated,
		incompatible: is_incompatible,
		deprecated: game_mod.deprecated == Some(true),
		has_remote_config,
		show_status: can_show_actions,
		can_install: can_show_actions
			&& !installed
			&& !info.is_outdated
			&& game_mod.install.is_some(),
		can_update: can_show_actions && info.is_outdated,
		can_run: can_show_actions
			&& game_mod.run_for_game.is_some()
			&& (game_mod.install.is_none() || installed),
		can_show_actions,
		can_edit_config: can_show_actions
			&& installed
			&& (game_mod.config.is_some() || has_remote_config),
		can_open_mod_folder: can_show_actions && installed,
		can_download_config: can_show_actions && installed && has_remote_config,
		can_reinstall: can_show_actions && installed && game_mod.install.is_some(),
		can_uninstall: can_show_actions && installed && game_mod.install.is_some(),
		has_installed_dependants: info.has_installed_dependants,
	}
}

fn sort_parts(parts: &mut [(&GameMod, &GameModInfo)]) {
	parts.sort_by(|(a, _), (b, _)| {
		let a_date = mod_release_date(a).unwrap_or_default();
		let b_date = mod_release_date(b).unwrap_or_default();

		b_date
			.cmp(a_date)
			.then_with(|| a.title.to_lowercase().cmp(&b.title.to_lowercase()))
	});
}

fn version_text(info: &GameModInfo, game_mod: &GameMod) -> String {
	let download_id = game_mod
		.download
		.as_ref()
		.map(|download| download.id.as_str());

	let mut parts = Vec::new();

	if let Some(version) = info.installed_version.as_deref().or(download_id) {
		parts.push(version);
	}

	if info.is_outdated
		&& let Some(download_id) = download_id
	{
		parts.push(download_id);
	}

	parts.join(" ➔ ")
}

pub(crate) fn release_date(game_mod: &GameMod) -> String {
	let Some(date) = mod_release_date(game_mod) else {
		return String::new();
	};

	let (day, month, year) = crate::model::format_release_date(Some(date));

	if year.is_empty() {
		return String::new();
	}

	format!("{month} {day}, {year}")
}

pub(crate) fn mod_release_date(game_mod: &GameMod) -> Option<&str> {
	game_mod
		.download
		.as_ref()
		.and_then(|download| download.release_date.as_deref())
}

pub(crate) fn engine_text(game_mod: &GameMod) -> String {
	let mut parts = Vec::new();

	if let Some(engine) = game_mod.engine {
		parts.push(engine.to_string());
	}

	if let Some(range) = &game_mod.engine_version_range
		&& let Some(range) = version_range(range)
	{
		parts.push(range);
	}

	if let Some(unity_backend) = game_mod.unity_backend {
		parts.push(unity_backend.to_string());
	}

	if let Some(game_os) = game_mod.game_os {
		parts.push(game_os.to_string());
	}

	if let Some(architecture) = game_mod.architecture {
		parts.push(architecture.to_string());
	}

	parts.join(" ")
}

fn version_range(range: &EngineVersionRange) -> Option<String> {
	let versions = [range.minimum.as_ref(), range.maximum.as_ref()]
		.into_iter()
		.flatten()
		.map(version_numbers)
		.collect::<Vec<_>>();

	if versions.is_empty() {
		None
	} else {
		Some(versions.join(" - "))
	}
}

fn version_numbers(version: &EngineVersionNumbers) -> String {
	let mut parts = vec![version.major.to_string()];

	if let Some(minor) = version.minor {
		parts.push(minor.to_string());
	}

	if let Some(patch) = version.patch {
		parts.push(patch.to_string());
	}

	parts.join(".")
}

fn build_potential_rows(
	compatible_rows: &[GameModRowData],
	families: &BTreeMap<String, ModFamily>,
) -> Vec<PotentialModRowData> {
	let mut family_counts = BTreeMap::<&str, usize>::new();

	for row in compatible_rows {
		if !row.family.is_empty() {
			*family_counts.entry(row.family.as_str()).or_default() += 1;
		}
	}

	let mut seen_families = HashSet::new();
	let mut result = Vec::new();

	for row in compatible_rows {
		let family = if !row.family.is_empty()
			&& family_counts.get(row.family.as_str()).copied().unwrap_or(0) > 1
		{
			families.get(row.family.as_str())
		} else {
			None
		};

		if let Some(family) = family {
			if seen_families.insert(row.family.to_string()) {
				result.push(PotentialModRowData {
					title: family.title.clone().into(),
					description: family.description.clone().into(),
				});
			}
		} else {
			result.push(PotentialModRowData {
				title: row.title.clone(),
				description: row.description.clone(),
			});
		}
	}

	result
}

fn collect_install_mods(
	database: &DbMutex,
	game: &DbGame,
	mod_id: &str,
) -> Result<(GameMod, Vec<GameMod>)> {
	let game_mod = database.get_mod(mod_id)?;
	let relevant_mods = database.get_game_mods(&game.provider_id, &game.game_id)?;
	let mut deps_to_install = Vec::new();

	collect_deps_to_install(
		mod_id,
		database,
		&relevant_mods,
		&mut HashSet::new(),
		&mut deps_to_install,
	);

	Ok((game_mod, deps_to_install))
}

async fn install_mod_inner(
	database: &DbMutex,
	game: &DbGame,
	game_mod: GameMod,
	deps_to_install: Vec<GameMod>,
	downloads: &Tracker,
) -> Result {
	let mod_id = game_mod.id.clone();

	for dependency in &deps_to_install {
		dependency
			.install(Some(game), |status| downloads.record(status))
			.await?;
	}

	if let Some(installed_mod) =
		database.get_installed_mod(&mod_id, Some(game.provider_id), Some(game.game_id.clone()))?
	{
		installed_mod.uninstall().await?;
	}

	if game_mod.install.is_some() {
		game_mod
			.install(Some(game), |status| downloads.record(status))
			.await?;
	}

	database.refresh_installed_mods()?;

	Ok(())
}

fn collect_deps_to_install(
	mod_id: &str,
	database: &DbMutex,
	relevant_mods: &[GameModInfo],
	visited: &mut HashSet<String>,
	result: &mut Vec<GameMod>,
) {
	if !visited.insert(mod_id.to_string()) {
		return;
	}

	let Ok(game_mod) = database.get_mod(mod_id) else {
		return;
	};

	for dependencies in [
		game_mod.optional_dependencies.as_ref(),
		game_mod.required_dependencies.as_ref(),
	]
	.into_iter()
	.flatten()
	{
		for dependency in dependencies {
			let matching_mods = match dependency {
				ModDependency::ModId { mod_id: dep_id } => relevant_mods
					.iter()
					.filter(|info| info.compatible && info.mod_id == *dep_id)
					.collect::<Vec<_>>(),
				ModDependency::Family { family: dep_family } => relevant_mods
					.iter()
					.filter(|info| {
						info.compatible && info.family.as_deref() == Some(dep_family.as_str())
					})
					.collect::<Vec<_>>(),
			};

			for info in matching_mods {
				let outdated = info.installed_version.is_none() || info.is_outdated;

				if outdated
					&& let Ok(dep_mod) = database.get_mod(&info.mod_id)
					&& dep_mod.install.is_some()
				{
					collect_deps_to_install(&info.mod_id, database, relevant_mods, visited, result);
					result.push(dep_mod);
				}
			}
		}
	}
}

fn refresh_after_action(window: &slint::Weak<MainWindow>, database: &Arc<DbMutex>, game: &DbGame) {
	let window = window.clone();
	let database = database.clone();
	let game = game.clone();

	let _ = slint::invoke_from_event_loop(move || {
		let Some(window) = window.upgrade() else {
			return;
		};

		if !window.get_game_selected()
			|| window.get_selected_game_row().game_id.to_string() != game.game_id
		{
			return;
		}

		refresh_mods(&window, &database, &game);
	});
}
