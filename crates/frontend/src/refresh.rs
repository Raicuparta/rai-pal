use std::{sync::Arc, thread, time::Duration};

use rai_pal_backend::{
	game_providers::game_provider::GameProviderId,
	games_query::GamesQuery,
	local_database::{
		app_database::DbMutex,
		game_database::{GameDatabase, attach_remote},
		mod_database::ModDatabase,
	},
	mod_providers::mod_provider,
	remote_game,
	result::LogErrExt,
};
use slint::ComponentHandle;

use crate::{
	MainWindow, mods, mods_tab, notifications, query::start_query, runtime,
	scheduler::FetchScheduler, selected,
};

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

		for provider in GameProviderId::ALL {
			push_task(&window, &mut tasks, &format!("{provider}"));

			if let Err(err) = provider.insert_games(&database) {
				log::error!("Failed to refresh {provider} games: {err}");
				notifications::error_async(
					&window,
					format!("Failed to refresh {provider} games: {err}"),
				);
			}

			pop_task(&window, &mut tasks, &format!("{provider}"));
		}

		if let Err(err) = database.refresh_installed_mods() {
			log::error!("Failed to refresh installed mods: {err}");
			notifications::error_async(&window, format!("Failed to refresh installed mods: {err}"));
		}

		push_task(&window, &mut tasks, "mods");
		if let Err(err) = refresh_mods(&database) {
			log::error!("Failed to refresh mods: {err}");
			notifications::error_async(&window, format!("Failed to refresh mods: {err}"));
		}
		pop_task(&window, &mut tasks, "mods");

		push_task(&window, &mut tasks, "remote data");
		if let Err(err) = refresh_remote_database() {
			log::error!("Failed to refresh remote database: {err}");
			notifications::error_async(
				&window,
				format!("Failed to refresh remote database: {err}"),
			);
		}
		pop_task(&window, &mut tasks, "remote data");

		crate::runtime::invoke_on_ui(move || {
			let Some(window) = window.upgrade() else {
				return;
			};

			window.global::<crate::Refresh>().set_refreshing(false);
			window.global::<crate::Refresh>().set_task_names("".into());
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

pub(crate) fn refresh_mods(database: &DbMutex) -> Result<(), String> {
	runtime::block_on(mod_provider::refresh_all_mods(database))
}

fn refresh_remote_database() -> Result<(), String> {
	runtime::block_on(async {
		let path = remote_game::download_database().await?;
		attach_remote(&path)
	})
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

	crate::runtime::invoke_on_ui(move || {
		let Some(window) = window.upgrade() else {
			return;
		};

		window.global::<crate::Refresh>().set_refreshing(active);
		window
			.global::<crate::Refresh>()
			.set_task_names(names.into());
	});
}

pub(crate) fn refresh_selected_game(
	window: &MainWindow,
	database: &Arc<DbMutex>,
	selected_game: &SelectedGameState,
) {
	let Some((_, game)) = selected_game
		.lock()
		.ok_or_log("Failed to lock selected game")
		.and_then(|game| game.clone())
	else {
		return;
	};

	let Some(updated_game) = database
		.get_game(&game.provider_id, &game.game_id)
		.ok_or_log("Failed to reload selected game")
	else {
		return;
	};

	let mut row = window.get_selected_game_row();
	crate::model::apply_game_to_row(&mut row, &updated_game);
	selected::show(window, &updated_game, row);
	mods::refresh_mods(window, database, &updated_game);
}
