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

const CATEGORIES: [&str; 4] = ["Installed", "Available", "Incompatible", "Other"];

pub fn refresh_mods(window: &MainWindow, database: &DbMutex, game: &DbGame) {
	let rows = load_rows(database, game);
	let has_installed_mods = rows.iter().any(|row| row.installed && !row.incompatible);

	window.set_has_installed_mods(has_installed_mods);
	window.set_game_mods(Rc::new(slint::VecModel::from(rows)).into());
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
			game.uninstall_all_mods().await?;
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

	let mut categories: [Vec<GameModRowData>; 4] = Default::default();

	for info in mod_infos {
		let Some(game_mod) = mods.get(&info.mod_id) else {
			continue;
		};

		let installed = info.installed_hash.is_some();
		let incompatible = !info.compatible;
		let category_index = if incompatible {
			2
		} else if installed {
			0
		} else if game_mod.hide_from_game_mods_list == Some(true) {
			3
		} else {
			1
		};

		let row = GameModRowData {
			mod_id: game_mod.id.clone().into(),
			title: game_mod.title.clone().into(),
			description: game_mod.description.clone().into(),
			version: version_label(&info, game_mod).into(),
			category: CATEGORIES[category_index].into(),
			show_category_header: false,
			installed,
			outdated: info.is_outdated,
			incompatible,
			can_install: !incompatible
				&& !installed
				&& game.exe_path.is_some()
				&& game_mod.install.is_some(),
			can_uninstall: installed && !info.has_installed_dependants,
			can_update: installed && info.is_outdated && game.exe_path.is_some(),
			can_run: !incompatible
				&& game.exe_path.is_some()
				&& game_mod.run_for_game.is_some()
				&& (game_mod.install.is_none() || installed),
			has_config: game_mod.config.is_some(),
			has_installed_dependants: info.has_installed_dependants,
		};

		categories[category_index].push(row);
	}

	let mut rows = Vec::new();

	for mut category_rows in categories {
		category_rows.sort_by(|a, b| a.title.cmp(&b.title));

		for (index, mut row) in category_rows.into_iter().enumerate() {
			row.show_category_header = index == 0;
			rows.push(row);
		}
	}

	rows
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
