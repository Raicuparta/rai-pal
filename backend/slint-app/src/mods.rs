use std::{collections::HashSet, rc::Rc, sync::Arc, thread};

use rai_pal_core::{
	game::DbGame,
	local_database::{
		app_database::DbMutex,
		mod_database::{GameModInfo, ModDatabase},
	},
	mods::game_mod::{GameMod, ModDependency},
	result::Result,
};

use crate::{GameModRowData, MainWindow};

pub fn refresh_mods(window: &MainWindow, database: &DbMutex, game: &DbGame) {
	let rows = load_rows(database, game);
	let has_installed_mods = game.exe_path.is_some()
		&& rows
			.iter()
			.any(|row| row.installed && row.section.as_str() != "incompatible");

	let mut compatible = Vec::new();
	let mut hidden = Vec::new();
	let mut incompatible = Vec::new();

	for row in rows {
		match row.section.as_str() {
			"hidden" => hidden.push(row),
			"incompatible" => incompatible.push(row),
			_ => compatible.push(row),
		}
	}

	window.set_has_installed_mods(has_installed_mods);
	window.set_has_compatible_mods(!compatible.is_empty());
	window.set_has_hidden_mods(!hidden.is_empty());
	window.set_has_incompatible_mods(!incompatible.is_empty());
	window.set_game_mods(Rc::new(slint::VecModel::from(compatible)).into());
	window.set_game_mods_hidden(Rc::new(slint::VecModel::from(hidden)).into());
	window.set_game_mods_incompatible(Rc::new(slint::VecModel::from(incompatible)).into());
}

pub fn install_mod(
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

		if let Err(err) = runtime.block_on(install_mod_inner(&database, &game, &mod_id)) {
			eprintln!("Failed to install mod {mod_id}: {err}");
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

fn load_rows(database: &DbMutex, game: &DbGame) -> Vec<GameModRowData> {
	let Ok(mod_infos) = database.get_game_mods(&game.provider_id, &game.game_id) else {
		return Vec::new();
	};
	let Ok(mods) = database.get_mod_map() else {
		return Vec::new();
	};

	let mut compatible = Vec::new();
	let mut hidden = Vec::new();
	let mut incompatible = Vec::new();

	for info in mod_infos {
		let Some(game_mod) = mods.get(&info.mod_id) else {
			continue;
		};

		let installed = info.installed_hash.is_some();
		let is_incompatible = !info.compatible;
		let can_show_actions = !is_incompatible && game.exe_path.is_some();

		let row = GameModRowData {
			mod_id: game_mod.id.clone().into(),
			title: game_mod.title.clone().into(),
			description: game_mod.description.clone().into(),
			version: version_label(&info, game_mod).into(),
			section: if is_incompatible {
				"incompatible".into()
			} else if game_mod.hide_from_game_mods_list == Some(true) {
				"hidden".into()
			} else {
				"compatible".into()
			},
			installed,
			outdated: info.is_outdated,
			incompatible: is_incompatible,
			deprecated: game_mod.deprecated == Some(true),
			show_status: can_show_actions,
			can_install: can_show_actions
				&& !installed
				&& !info.is_outdated
				&& game_mod.install.is_some(),
			can_uninstall: can_show_actions && installed && game_mod.install.is_some(),
			can_update: can_show_actions && installed && info.is_outdated,
			can_run: can_show_actions
				&& game_mod.run_for_game.is_some()
				&& (game_mod.install.is_none() || installed),
			can_show_actions,
			has_config: game_mod.config.is_some(),
			has_installed_dependants: info.has_installed_dependants,
		};

		if is_incompatible {
			incompatible.push(row);
		} else if game_mod.hide_from_game_mods_list == Some(true) {
			hidden.push(row);
		} else {
			compatible.push(row);
		}
	}

	compatible.sort_by(|a, b| a.title.to_lowercase().cmp(&b.title.to_lowercase()));
	hidden.sort_by(|a, b| a.title.to_lowercase().cmp(&b.title.to_lowercase()));
	incompatible.sort_by(|a, b| a.title.to_lowercase().cmp(&b.title.to_lowercase()));

	compatible
		.into_iter()
		.chain(hidden)
		.chain(incompatible)
		.collect()
}

fn version_label(info: &GameModInfo, game_mod: &GameMod) -> String {
	let raw = info
		.installed_version
		.clone()
		.or_else(|| {
			game_mod
				.download
				.as_ref()
				.map(|download| download.id.clone())
		})
		.unwrap_or_else(|| "-".to_string());

	raw.split('/').next().unwrap_or("-").to_string()
}

async fn install_mod_inner(database: &DbMutex, game: &DbGame, mod_id: &str) -> Result {
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

	for dependency in &deps_to_install {
		dependency.install(Some(game), |_| {}).await?;
	}

	if let Some(installed_mod) =
		database.get_installed_mod(mod_id, Some(game.provider_id), Some(game.game_id.clone()))?
	{
		installed_mod.uninstall().await?;
	}

	if game_mod.install.is_some() {
		game_mod.install(Some(game), |_| {}).await?;
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
