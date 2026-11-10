use std::{
	collections::BTreeMap,
	path::PathBuf,
	rc::Rc,
	sync::{
		Arc,
		atomic::{AtomicU64, Ordering},
	},
	thread,
	time::Duration,
};

use rai_pal_backend::{
	game::Game,
	game_providers::{
		game_provider::GameProviderId, manual_provider, provider_command::ProviderCommandAction,
	},
	local_database::{game_database::GameDatabase, mod_database::ModDatabase},
	path_extensions::PathExt,
};
use slint::ComponentHandle;

use crate::{AddGamePhase, MainWindow, notifications, runtime, selected};

use super::{CommandHandler, on_command};

static SCAN_GENERATION: AtomicU64 = AtomicU64::new(0);

impl CommandHandler {
	pub fn show_add_game(&self) {
		let Some(window) = self.ui() else {
			return;
		};

		window.set_current_tab(crate::AppTab::Games);
		window
			.global::<crate::AppActions>()
			.invoke_tab_changed(crate::AppTab::Games);
		window.global::<crate::AddGameState>().set_open(true);
		reset_scan(&window);
		reload_directories(&window);
	}

	pub fn cancel_game_scan(&self) {
		SCAN_GENERATION.fetch_add(1, Ordering::SeqCst);

		if let Some(window) = self.ui() {
			reset_scan(&window);
		}
	}

	pub fn select_executable(&self) {
		let window_weak = self.window();
		let database = Arc::clone(&self.database);

		thread::spawn(move || {
			let Some(path) = pick_executable() else {
				return;
			};
			let path = path.normalize();

			let result = runtime::block_on(async {
				let game = manual_provider::add_game(&path).await?;
				database.insert_game(&game);
				Ok::<(), rai_pal_backend::result::Error>(())
			});

			crate::runtime::invoke_on_ui(move || {
				let Some(window) = window_weak.upgrade() else {
					return;
				};

				match result {
					Ok(()) => {
						window.global::<crate::AddGameState>().set_open(false);

						if let Some(handler) = Self::current() {
							handler.refresh_after_change();
						}
					}
					Err(err) => {
						notifications::error(&window, err);
					}
				}
			});
		});
	}

	pub fn select_directory(&self) {
		let Some(main_window) = self.ui() else {
			return;
		};

		let generation = SCAN_GENERATION.fetch_add(1, Ordering::SeqCst) + 1;
		let window_weak = main_window.as_weak();
		let progress_window = main_window.as_weak();

		thread::spawn(move || {
			let Some(path) = pick_directory() else {
				return;
			};
			let path = path.normalize();
			let path_text = path.display().to_string();

			crate::runtime::invoke_on_ui({
				let window = window_weak.clone();
				let path_text = path_text.clone();

				move || {
					let Some(window) = window.upgrade() else {
						return;
					};

					reset_scan(&window);
					window
						.global::<crate::AddGameState>()
						.set_phase(AddGamePhase::Scanning);
					window
						.global::<crate::AddGameState>()
						.set_selected_path(path_text.into());
				}
			});

			let result = manual_provider::scan_directory(&path, move |progress| {
				if SCAN_GENERATION.load(Ordering::SeqCst) != generation {
					return;
				}

				let window = progress_window.clone();

				crate::runtime::invoke_on_ui(move || {
					if SCAN_GENERATION.load(Ordering::SeqCst) != generation {
						return;
					}

					let Some(window) = window.upgrade() else {
						return;
					};

					window
						.global::<crate::AddGameState>()
						.set_scanned_dirs(i32::try_from(progress.scanned_dirs).unwrap_or(i32::MAX));
					window
						.global::<crate::AddGameState>()
						.set_executables_found(
							i32::try_from(progress.executables_found).unwrap_or(i32::MAX),
						);
					window
						.global::<crate::AddGameState>()
						.set_current_path(progress.current_path.into());
				});
			});

			crate::runtime::invoke_on_ui(move || {
				if SCAN_GENERATION.load(Ordering::SeqCst) != generation {
					return;
				}

				let Some(window) = window_weak.upgrade() else {
					return;
				};

				match result {
					Ok(result) => {
						window
							.global::<crate::AddGameState>()
							.set_games_count(i32::try_from(result.games.len()).unwrap_or(i32::MAX));
						window
							.global::<crate::AddGameState>()
							.set_duration_text(format!("{:.1}", result.duration_secs).into());
						window
							.global::<crate::AddGameState>()
							.set_directory(path_text.into());
						window
							.global::<crate::AddGameState>()
							.set_phase(AddGamePhase::Confirming);
					}
					Err(err) => {
						reset_scan(&window);
						notifications::error(&window, err.to_string());
					}
				}
			});
		});
	}

	pub fn confirm_add_directory(&self) {
		let Some(main_window) = self.ui() else {
			return;
		};

		let path = PathBuf::from(
			main_window
				.global::<crate::AddGameState>()
				.get_directory()
				.to_string(),
		);

		if path.as_os_str().is_empty() {
			return;
		}

		let window_weak = main_window.as_weak();
		let database = Arc::clone(&self.database);

		thread::spawn(move || {
			let result = runtime::block_on(async {
				let games = manual_provider::add_directory(&path).await?;

				for game in &games {
					database.insert_game(game);
				}

				Ok::<(), rai_pal_backend::result::Error>(())
			});

			crate::runtime::invoke_on_ui(move || {
				let Some(window) = window_weak.upgrade() else {
					return;
				};

				reset_scan(&window);

				match result {
					Ok(()) => {
						reload_directories(&window);

						if let Some(handler) = Self::current() {
							handler.refresh_after_change();
						}
					}
					Err(err) => {
						notifications::error(&window, err);
					}
				}
			});
		});
	}

	pub fn remove_directory(&self, path: String) {
		let window_weak = self.window();
		let database = Arc::clone(&self.database);

		thread::spawn(move || {
			let path = PathBuf::from(path);

			let result = runtime::block_on(async {
				manual_provider::remove_directory(&path).await?;
				GameProviderId::Manual.insert_games(&database)?;
				database.refresh_installed_mods()?;
				Ok::<(), rai_pal_backend::result::Error>(())
			});

			crate::runtime::invoke_on_ui(move || {
				let Some(window) = window_weak.upgrade() else {
					return;
				};

				match result {
					Ok(()) => {
						reload_directories(&window);

						if let Some(handler) = Self::current() {
							handler.refresh_after_change();
						}
					}
					Err(err) => {
						notifications::error(&window, err);
					}
				}
			});
		});
	}

	pub fn remove_game(&self, game: Game) {
		let window_weak = self.window();
		let database = Arc::clone(&self.database);

		thread::spawn(move || {
			let result = runtime::block_on(async {
				game.get_provider_command(ProviderCommandAction::Delete)
					.ok_or_else(|| {
						rai_pal_backend::result::Error::DataEntryNotFound(
							ProviderCommandAction::Delete.to_string(),
						)
					})?
					.run(&game, &BTreeMap::default(), &[])?;
				game.provider_id.insert_games(&database)?;
				database.refresh_installed_mods()?;
				Ok::<(), rai_pal_backend::result::Error>(())
			});

			crate::runtime::invoke_on_ui(move || {
				let Some(window) = window_weak.upgrade() else {
					return;
				};

				match result {
					Ok(()) => {
						if let Some(handler) = Self::current() {
							handler.clear_selected_game();
							selected::hide(&window);
							handler.refresh_after_change();
						}
					}
					Err(err) => {
						notifications::error(&window, err);
					}
				}
			});
		});
	}

	pub fn add_dropped_paths(&self, paths: Vec<PathBuf>) {
		let window_weak = self.window();
		let database = Arc::clone(&self.database);

		thread::spawn(move || {
			let result = runtime::block_on(async {
				for path in &paths {
					let path = path.normalize();

					if path.is_dir() {
						for game in manual_provider::add_directory(&path).await? {
							database.insert_game(&game);
						}
					} else {
						let game = manual_provider::add_game(&path).await?;
						database.insert_game(&game);
					}
				}

				Ok::<(), rai_pal_backend::result::Error>(())
			});

			crate::runtime::invoke_on_ui(move || {
				let Some(window) = window_weak.upgrade() else {
					return;
				};

				match result {
					Ok(()) => {
						if let Some(handler) = Self::current() {
							handler.refresh_after_change();
						}
					}
					Err(err) => {
						notifications::error(&window, err);
					}
				}
			});
		});
	}

	fn refresh_after_change(&self) {
		self.load_mods_tab();
		self.start_query(Duration::ZERO);
	}
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
	window
		.global::<crate::AddGameState>()
		.set_phase(AddGamePhase::Idle);
	window
		.global::<crate::AddGameState>()
		.set_selected_path("".into());
	window.global::<crate::AddGameState>().set_scanned_dirs(0);
	window
		.global::<crate::AddGameState>()
		.set_executables_found(0);
	window
		.global::<crate::AddGameState>()
		.set_current_path("".into());
	window.global::<crate::AddGameState>().set_games_count(0);
	window
		.global::<crate::AddGameState>()
		.set_duration_text("".into());
	window
		.global::<crate::AddGameState>()
		.set_directory("".into());
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

	window
		.global::<crate::AddGameState>()
		.set_manual_directories(Rc::new(slint::VecModel::from(directories)).into());
}

pub fn register(window: &crate::MainWindow) {
	on_command!(window, DialogActions, on_request_add_game, || handler => {
		handler.show_add_game();
	});
	on_command!(window, DialogActions, on_add_game_closed, || handler => {
		handler.cancel_game_scan();
	});
	on_command!(window, DialogActions, on_select_game_executable, || handler => {
		handler.select_executable();
	});
	on_command!(window, DialogActions, on_select_game_directory, || handler => {
		handler.select_directory();
	});
	on_command!(window, DialogActions, on_cancel_game_scan, || handler => {
		handler.cancel_game_scan();
	});
	on_command!(window, DialogActions, on_confirm_game_directory, || handler => {
		handler.confirm_add_directory();
	});
	on_command!(window, DialogActions, on_remove_game_directory, |path| handler => {
		handler.remove_directory(path.to_string());
	});
}
