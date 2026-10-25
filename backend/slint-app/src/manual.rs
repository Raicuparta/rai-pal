use std::{
	path::PathBuf,
	rc::Rc,
	sync::{
		Arc,
		atomic::{AtomicU64, Ordering},
	},
	thread,
	time::Duration,
};

use rai_pal_core::{
	game::DbGame,
	game_providers::{game_provider::GameProviderId, manual_provider},
	games_query::GamesQuery,
	local_database::{
		app_database::DbMutex, game_database::GameDatabase, mod_database::ModDatabase,
	},
	path_extensions::PathExt,
};
use slint::ComponentHandle;

use crate::{
	AddGamePhase, MainWindow, mods_tab, notifications, query::start_query,
	scheduler::FetchScheduler, selected,
};

static SCAN_GENERATION: AtomicU64 = AtomicU64::new(0);

pub fn show(window: &MainWindow) {
	reset_scan(window);
	reload_directories(window);
	window.set_add_game_open(true);
}

pub fn cancel_scan(window: &MainWindow) {
	SCAN_GENERATION.fetch_add(1, Ordering::SeqCst);
	reset_scan(window);
}

pub fn select_executable(
	window: &MainWindow,
	database: Arc<DbMutex>,
	scheduler: Arc<FetchScheduler>,
	query: GamesQuery,
) {
	let window_weak = window.as_weak();

	thread::spawn(move || {
		let Some(path) = pick_executable() else {
			return;
		};
		let path = path.normalize();

		let result = run_async(async {
			let game = manual_provider::add_game(&path).await?;
			database.insert_game(&game);
			Ok::<(), rai_pal_core::result::Error>(())
		});

		let _ = slint::invoke_from_event_loop(move || {
			let Some(window) = window_weak.upgrade() else {
				return;
			};

			match result {
				Ok(()) => {
					window.set_add_game_open(false);
					refresh_after_change(&window, database, scheduler, query);
				}
				Err(err) => {
					notifications::error(&window, err);
				}
			}
		});
	});
}

pub fn select_directory(window: &MainWindow) {
	let generation = SCAN_GENERATION.fetch_add(1, Ordering::SeqCst) + 1;
	let window_weak = window.as_weak();
	let progress_window = window.as_weak();

	thread::spawn(move || {
		let Some(path) = pick_directory() else {
			return;
		};
		let path = path.normalize();
		let path_text = path.display().to_string();

		let _ = slint::invoke_from_event_loop({
			let window = window_weak.clone();
			let path_text = path_text.clone();

			move || {
				let Some(window) = window.upgrade() else {
					return;
				};

				reset_scan(&window);
				window.set_scan_phase(AddGamePhase::Scanning);
				window.set_scan_selected_path(path_text.into());
			}
		});

		let result = manual_provider::scan_directory(&path, move |progress| {
			if SCAN_GENERATION.load(Ordering::SeqCst) != generation {
				return;
			}

			let window = progress_window.clone();

			let _ = slint::invoke_from_event_loop(move || {
				if SCAN_GENERATION.load(Ordering::SeqCst) != generation {
					return;
				}

				let Some(window) = window.upgrade() else {
					return;
				};

				window.set_scan_scanned_dirs(progress.scanned_dirs as i32);
				window.set_scan_executables_found(progress.executables_found as i32);
				window.set_scan_current_path(progress.current_path.into());
			});
		});

		let _ = slint::invoke_from_event_loop(move || {
			if SCAN_GENERATION.load(Ordering::SeqCst) != generation {
				return;
			}

			let Some(window) = window_weak.upgrade() else {
				return;
			};

			match result {
				Ok(result) => {
					window.set_scan_games_count(result.games.len() as i32);
					window.set_scan_duration_text(format!("{:.1}", result.duration_secs).into());
					window.set_add_game_directory(path_text.into());
					window.set_scan_phase(AddGamePhase::Confirming);
				}
				Err(err) => {
					reset_scan(&window);
					notifications::error(&window, err.to_string());
				}
			}
		});
	});
}

pub fn confirm_add_directory(
	window: &MainWindow,
	database: Arc<DbMutex>,
	scheduler: Arc<FetchScheduler>,
	query: GamesQuery,
) {
	let path = PathBuf::from(window.get_add_game_directory().to_string());

	if path.as_os_str().is_empty() {
		return;
	}

	let window_weak = window.as_weak();

	thread::spawn(move || {
		let result = run_async(async {
			let games = manual_provider::add_directory(&path).await?;

			for game in &games {
				database.insert_game(game);
			}

			Ok::<(), rai_pal_core::result::Error>(())
		});

		let _ = slint::invoke_from_event_loop(move || {
			let Some(window) = window_weak.upgrade() else {
				return;
			};

			reset_scan(&window);

			match result {
				Ok(()) => {
					reload_directories(&window);
					refresh_after_change(&window, database, scheduler, query);
				}
				Err(err) => {
					notifications::error(&window, err);
				}
			}
		});
	});
}

pub fn remove_directory(
	window: &MainWindow,
	database: Arc<DbMutex>,
	scheduler: Arc<FetchScheduler>,
	query: GamesQuery,
	path: String,
) {
	let window_weak = window.as_weak();

	thread::spawn(move || {
		let path = PathBuf::from(path);

		let result = run_async(async {
			manual_provider::remove_directory(&path).await?;
			GameProviderId::Manual.insert_games(&database)?;
			database.refresh_installed_mods()?;
			Ok::<(), rai_pal_core::result::Error>(())
		});

		let _ = slint::invoke_from_event_loop(move || {
			let Some(window) = window_weak.upgrade() else {
				return;
			};

			match result {
				Ok(()) => {
					reload_directories(&window);
					refresh_after_change(&window, database, scheduler, query);
				}
				Err(err) => {
					notifications::error(&window, err);
				}
			}
		});
	});
}

pub fn remove_game(
	window: &MainWindow,
	database: Arc<DbMutex>,
	scheduler: Arc<FetchScheduler>,
	query: GamesQuery,
	selected_game: crate::selected::SelectedGameState,
	game: DbGame,
) {
	let window_weak = window.as_weak();

	thread::spawn(move || {
		let result = run_async(async {
			manual_provider::remove_game(&game).await?;
			GameProviderId::Manual.insert_games(&database)?;
			database.refresh_installed_mods()?;
			Ok::<(), rai_pal_core::result::Error>(())
		});

		let _ = slint::invoke_from_event_loop(move || {
			let Some(window) = window_weak.upgrade() else {
				return;
			};

			match result {
				Ok(()) => {
					if let Ok(mut selected_game) = selected_game.lock() {
						*selected_game = None;
					}

					selected::hide(&window);
					refresh_after_change(&window, database, scheduler, query);
				}
				Err(err) => {
					notifications::error(&window, err);
				}
			}
		});
	});
}

fn pick_executable() -> Option<PathBuf> {
	rfd::FileDialog::new()
		.set_title("Select the game executable")
		.add_filter("Windows executable", &["exe"])
		.add_filter("Other executable", &["*"])
		.pick_file()
}

fn pick_directory() -> Option<PathBuf> {
	rfd::FileDialog::new()
		.set_title("Select a game directory")
		.pick_folder()
}

fn reset_scan(window: &MainWindow) {
	window.set_scan_phase(AddGamePhase::Idle);
	window.set_scan_selected_path("".into());
	window.set_scan_scanned_dirs(0);
	window.set_scan_executables_found(0);
	window.set_scan_current_path("".into());
	window.set_scan_games_count(0);
	window.set_scan_duration_text("".into());
	window.set_add_game_directory("".into());
}

fn reload_directories(window: &MainWindow) {
	let directories = match manual_provider::get_directories() {
		Ok(directories) => directories
			.into_iter()
			.map(|path| path.display().to_string().into())
			.collect::<Vec<slint::SharedString>>(),
		Err(err) => {
			notifications::error(window, err.to_string());
			Vec::new()
		}
	};

	window.set_manual_directories(Rc::new(slint::VecModel::from(directories)).into());
}

fn refresh_after_change(
	window: &MainWindow,
	database: Arc<DbMutex>,
	scheduler: Arc<FetchScheduler>,
	query: GamesQuery,
) {
	mods_tab::load(window, &database);
	start_query(window.as_weak(), database, scheduler, query, Duration::ZERO);
}

fn run_async<T>(
	future: impl std::future::Future<Output = rai_pal_core::result::Result<T>>,
) -> Result<T, String> {
	let runtime = tokio::runtime::Builder::new_current_thread()
		.enable_all()
		.build()
		.map_err(|err| err.to_string())?;

	runtime.block_on(future).map_err(|err| err.to_string())
}
