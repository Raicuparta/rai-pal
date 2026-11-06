use std::{collections::HashSet, rc::Rc, sync::Arc, thread};

use rai_pal_core::{
	game::Game,
	game_engines::game_engine::{EngineVersionNumbers, EngineVersionRange},
	local_database::{
		app_database::DbMutex,
		mod_database::{ModDatabase, ModInfo},
	},
	mods::{
		game_mods::{GameMod, GameModGroups, PotentialMod},
		manifest::{Mod, ModDependency, install_steps},
	},
	result::Result,
};
use slint::ComponentHandle;

use crate::{
	GameModView, MainWindow, PotentialModView, action_state, downloads::Tracker, notifications,
	runtime,
};

#[derive(Default)]
struct LoadedMods {
	installed: Vec<GameModView>,
	available: Vec<GameModView>,
	hidden: Vec<GameModView>,
	incompatible: Vec<GameModView>,
	potential: Vec<PotentialModView>,
	has_any_mods: bool,
}

pub fn refresh_mods(window: &MainWindow, database: &Arc<DbMutex>, game: &Game) {
	apply_mods(window, database, game, &HashSet::new());
	refresh_remote_configs(window.as_weak(), database.clone(), game.clone());
}

fn refresh_remote_configs(window: slint::Weak<MainWindow>, database: Arc<DbMutex>, game: Game) {
	if game.exe_path.is_none() {
		return;
	}

	thread::spawn(move || {
		let Ok(runtime) = runtime::current_thread() else {
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
	game: &Game,
	remote_config_ids: &HashSet<String>,
) {
	let loaded = match load_mods(database, game, remote_config_ids) {
		Ok(loaded) => loaded,
		Err(err) => {
			eprintln!("Failed to load mods: {err}");
			notifications::error(window, format!("Failed to load mods: {err}"));
			return;
		}
	};

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
	game: Game,
	mod_id: String,
	downloads: Tracker,
) {
	let action_id = format!("game-mod-install:{mod_id}");
	action_state::start(&window, action_id.clone());

	thread::spawn(move || {
		let Ok(runtime) = runtime::current_thread() else {
			return;
		};

		let (game_mod, deps_to_install) = match collect_install_mods(&database, &game, &mod_id) {
			Ok(mods) => mods,
			Err(err) => {
				eprintln!("Failed to prepare mod install {mod_id}: {err}");
				action_state::finish(
					&window,
					&action_id,
					Err(err),
					format!("Failed to install mod {mod_id}"),
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
			action_state::finish(
				&window,
				&action_id,
				Err(err),
				format!("Failed to install mod {mod_id}"),
			);
			return;
		}

		downloads.finish_steps(&steps);
		refresh_after_action(&window, &database, &game);
		action_state::finish(
			&window,
			&action_id,
			Ok::<(), rai_pal_core::result::Error>(()),
			"",
		);
	});
}

pub fn download_mod_config(
	database: Arc<DbMutex>,
	window: slint::Weak<MainWindow>,
	game: Game,
	mod_id: String,
) {
	let action_id = format!("game-mod-download-config:{mod_id}");
	action_state::start(&window, action_id.clone());

	thread::spawn(move || {
		let Ok(runtime) = runtime::current_thread() else {
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
			action_state::finish(
				&window,
				&action_id,
				Err(err),
				format!("Failed to download config for mod {mod_id}"),
			);
			return;
		}

		refresh_after_action(&window, &database, &game);
		action_state::finish(
			&window,
			&action_id,
			Ok::<(), rai_pal_core::result::Error>(()),
			"",
		);
	});
}

pub fn uninstall_mod(
	database: Arc<DbMutex>,
	window: slint::Weak<MainWindow>,
	game: Game,
	mod_id: String,
) {
	let action_id = format!("game-mod-uninstall:{mod_id}");
	action_state::start(&window, action_id.clone());

	thread::spawn(move || {
		let Ok(runtime) = runtime::current_thread() else {
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
			action_state::finish(
				&window,
				&action_id,
				Err(err),
				format!("Failed to uninstall mod {mod_id}"),
			);
			return;
		}

		refresh_after_action(&window, &database, &game);
		action_state::finish(
			&window,
			&action_id,
			Ok::<(), rai_pal_core::result::Error>(()),
			"",
		);
	});
}

pub fn uninstall_all_mods(database: Arc<DbMutex>, window: slint::Weak<MainWindow>, game: Game) {
	let action_id = "game-mod-uninstall-all".to_string();
	action_state::start(&window, action_id.clone());

	thread::spawn(move || {
		let Ok(runtime) = runtime::current_thread() else {
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
			action_state::finish(
				&window,
				&action_id,
				Err(err),
				"Failed to uninstall all mods",
			);
			return;
		}

		refresh_after_action(&window, &database, &game);
		action_state::finish(
			&window,
			&action_id,
			Ok::<(), rai_pal_core::result::Error>(()),
			"",
		);
	});
}

pub fn run_mod(
	database: Arc<DbMutex>,
	window: slint::Weak<MainWindow>,
	game: Game,
	mod_id: String,
) {
	let action_id = format!("game-mod-run:{mod_id}");
	action_state::start(&window, action_id.clone());

	thread::spawn(move || {
		let result = database
			.get_mod(&mod_id)
			.and_then(|game_mod| game_mod.run(Some(&game)));

		action_state::finish(
			&window,
			&action_id,
			result,
			format!("Failed to run mod {mod_id}"),
		);
	});
}

pub fn open_mod_folder(
	database: Arc<DbMutex>,
	window: slint::Weak<MainWindow>,
	game: Game,
	mod_id: String,
) {
	let action_id = format!("game-mod-open-folder:{mod_id}");
	action_state::start(&window, action_id.clone());

	thread::spawn(move || {
		let result = database
			.try_get_installed_mod(&game.provider_id, &game.game_id, &mod_id)
			.and_then(|installed_mod| installed_mod.open_folder());

		action_state::finish(
			&window,
			&action_id,
			result,
			format!("Failed to open mod folder {mod_id}"),
		);
	});
}

pub fn configure_mod(
	database: Arc<DbMutex>,
	window: slint::Weak<MainWindow>,
	game: Game,
	mod_id: String,
	open_folder: bool,
) {
	let action_id = if open_folder {
		format!("game-mod-configure-folder:{mod_id}")
	} else {
		format!("game-mod-configure:{mod_id}")
	};
	action_state::start(&window, action_id.clone());

	thread::spawn(move || {
		let result = database
			.try_get_installed_mod(&game.provider_id, &game.game_id, &mod_id)
			.and_then(|installed_mod| installed_mod.configure(open_folder));

		action_state::finish(
			&window,
			&action_id,
			result,
			format!("Failed to configure mod {mod_id}"),
		);
	});
}

fn load_mods(
	database: &DbMutex,
	game: &Game,
	remote_config_ids: &HashSet<String>,
) -> Result<LoadedMods> {
	let list = GameModGroups::load(database, game, remote_config_ids)?;

	let installed = list
		.installed
		.into_iter()
		.map(|entry| row_data(entry, game))
		.collect::<Vec<_>>();
	let available = list
		.available
		.into_iter()
		.map(|entry| row_data(entry, game))
		.collect::<Vec<_>>();
	let hidden = list
		.hidden
		.into_iter()
		.map(|entry| row_data(entry, game))
		.collect::<Vec<_>>();
	let incompatible = list
		.incompatible
		.into_iter()
		.map(|entry| row_data(entry, game))
		.collect::<Vec<_>>();
	let has_any_mods = !installed.is_empty() || !available.is_empty() || !incompatible.is_empty();

	Ok(LoadedMods {
		installed,
		available,
		hidden,
		incompatible,
		potential: list.potential.into_iter().map(potential_row_data).collect(),
		has_any_mods,
	})
}

fn row_data(entry: GameMod, game: &Game) -> GameModView {
	let GameMod {
		manifest,
		info,
		has_remote_config,
	} = entry;
	let installed = info.installed_hash.is_some();
	let is_incompatible = !info.compatible;
	let can_show_actions = !is_incompatible && game.exe_path.is_some();
	let version_text = version_text(&info, &manifest);
	let release_date = release_date_display(&manifest);
	let engine_text = engine_text(&manifest);

	GameModView {
		mod_id: manifest.id.into(),
		title: manifest.title.into(),
		description: manifest.description.into(),
		family: manifest.family.unwrap_or_default().into(),
		version_text: version_text.into(),
		release_date: release_date.into(),
		author: manifest.author.into(),
		engine_text: engine_text.into(),
		installed,
		outdated: info.is_outdated,
		incompatible: is_incompatible,
		deprecated: manifest.deprecated == Some(true),
		has_remote_config,
		show_status: can_show_actions,
		can_install: can_show_actions
			&& !installed
			&& !info.is_outdated
			&& manifest.install.is_some(),
		can_update: can_show_actions && info.is_outdated,
		can_run: can_show_actions
			&& manifest.run_for_game.is_some()
			&& (manifest.install.is_none() || installed),
		can_show_actions,
		can_edit_config: can_show_actions
			&& installed
			&& (manifest.config.is_some() || has_remote_config),
		can_open_mod_folder: can_show_actions && installed,
		can_download_config: can_show_actions && installed && has_remote_config,
		can_reinstall: can_show_actions && installed && manifest.install.is_some(),
		can_uninstall: can_show_actions && installed && manifest.install.is_some(),
		has_installed_dependants: info.has_installed_dependants,
	}
}

fn potential_row_data(item: PotentialMod) -> PotentialModView {
	let (title, description) = match item {
		PotentialMod::Family(family) => (family.title, family.description),
		PotentialMod::Manifest(manifest) => {
			let manifest = *manifest;
			(manifest.title, manifest.description)
		}
	};

	PotentialModView {
		title: title.into(),
		description: description.into(),
	}
}

fn version_text(info: &ModInfo, game_mod: &Mod) -> String {
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

pub(crate) fn release_date_display(game_mod: &Mod) -> String {
	crate::model::ReleaseDate::parse(game_mod.release_date()).display_short()
}

pub(crate) fn download_version(game_mod: &Mod) -> String {
	game_mod
		.download
		.as_ref()
		.map(|download| download.id.clone())
		.unwrap_or_default()
}

pub(crate) fn engine_text(game_mod: &Mod) -> String {
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

fn collect_install_mods(database: &DbMutex, game: &Game, mod_id: &str) -> Result<(Mod, Vec<Mod>)> {
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
	game: &Game,
	game_mod: Mod,
	deps_to_install: Vec<Mod>,
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
	relevant_mods: &[ModInfo],
	visited: &mut HashSet<String>,
	result: &mut Vec<Mod>,
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

fn refresh_after_action(window: &slint::Weak<MainWindow>, database: &Arc<DbMutex>, game: &Game) {
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
