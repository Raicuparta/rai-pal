use std::{sync::Arc, thread, time::Duration};

use rai_pal_core::{
	game_providers::game_provider::GameProviderId,
	games_query::GamesQuery,
	local_database::{
		app_database::DbMutex,
		game_database::{GameDatabase, attach_remote},
		mod_database::ModDatabase,
	},
	mod_providers::mod_provider,
	remote_game,
};
use slint::ComponentHandle;

use crate::{MainWindow, mods, mods_tab, query::start_query, scheduler::FetchScheduler, selected};

const PROVIDERS: &[GameProviderId] = &[
	GameProviderId::Amazon,
	GameProviderId::Epic,
	GameProviderId::Gog,
	GameProviderId::Itch,
	GameProviderId::Manual,
	GameProviderId::Steam,
	GameProviderId::Xbox,
];

type SelectedGameState = crate::selected::SelectedGameState;

pub fn refresh_all(
	window: slint::Weak<MainWindow>,
	database: Arc<DbMutex>,
	scheduler: Arc<FetchScheduler>,
	query: GamesQuery,
	selected_game: SelectedGameState,
) {
	thread::spawn(move || {
		let mut tasks = Vec::<String>::new();

		for provider in PROVIDERS {
			push_task(&window, &mut tasks, &format!("{provider}"));

			if let Err(err) = provider.insert_games(&database) {
				eprintln!("Failed to refresh {provider} games: {err}");
			}

			pop_task(&window, &mut tasks, &format!("{provider}"));
		}

		if let Err(err) = database.refresh_installed_mods() {
			eprintln!("Failed to refresh installed mods: {err}");
		}

		push_task(&window, &mut tasks, "mods");
		refresh_mods(&database);
		pop_task(&window, &mut tasks, "mods");

		push_task(&window, &mut tasks, "remote data");
		refresh_remote_database();
		pop_task(&window, &mut tasks, "remote data");

		let _ = slint::invoke_from_event_loop(move || {
			let Some(window) = window.upgrade() else {
				return;
			};

			window.set_refreshing(false);
			window.set_refresh_task_names("".into());
			mods_tab::load(&window, &database);
			start_query(
				window.as_weak(),
				database.clone(),
				scheduler,
				query,
				Duration::ZERO,
			);
			refresh_selected_game(&window, &database, &selected_game);
		});
	});
}

pub(crate) fn refresh_mods(database: &DbMutex) {
	let Ok(runtime) = tokio::runtime::Builder::new_current_thread()
		.enable_all()
		.build()
	else {
		return;
	};

	if let Err(err) = runtime.block_on(mod_provider::refresh_all_mods(database)) {
		eprintln!("Failed to refresh mods: {err}");
	}
}

fn refresh_remote_database() {
	let Ok(runtime) = tokio::runtime::Builder::new_current_thread()
		.enable_all()
		.build()
	else {
		return;
	};

	let result = runtime.block_on(async {
		let path = remote_game::download_database().await?;
		attach_remote(&path)
	});

	if let Err(err) = result {
		eprintln!("Failed to refresh remote database: {err}");
	}
}

fn push_task(window: &slint::Weak<MainWindow>, tasks: &mut Vec<String>, task: &str) {
	tasks.push(task.to_string());
	publish_tasks(window, tasks);
}

fn pop_task(window: &slint::Weak<MainWindow>, tasks: &mut Vec<String>, task: &str) {
	if let Some(index) = tasks.iter().position(|item| item == task) {
		tasks.remove(index);
	}
	publish_tasks(window, tasks);
}

fn publish_tasks(window: &slint::Weak<MainWindow>, tasks: &[String]) {
	let window = window.clone();
	let names = tasks.join(", ");
	let active = !tasks.is_empty();

	let _ = slint::invoke_from_event_loop(move || {
		let Some(window) = window.upgrade() else {
			return;
		};

		window.set_refreshing(active);
		window.set_refresh_task_names(names.into());
	});
}

pub(crate) fn refresh_selected_game(
	window: &MainWindow,
	database: &Arc<DbMutex>,
	selected_game: &SelectedGameState,
) {
	let Some((_, game)) = selected_game.lock().ok().and_then(|game| game.clone()) else {
		return;
	};

	let Ok(updated_game) = database.get_game(&game.provider_id, &game.game_id) else {
		return;
	};

	let mut row = window.get_selected_game_row();
	crate::model::apply_game_to_row(&mut row, &updated_game);
	selected::show(window, &updated_game, row);
	mods::refresh_mods(window, database, &updated_game);
}
