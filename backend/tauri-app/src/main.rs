// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
// Command stuff needs to be async so I can spawn tasks.
#![allow(clippy::unused_async)]

use std::{
	path::PathBuf,
	thread,
	time::Duration,
};

use app_settings::AppSettings;
use app_state::{
	AppState,
	StateData,
	StatefulHandle,
};
use events::EventEmitter;
#[cfg(target_os = "windows")]
use rai_pal_core::windows;
use rai_pal_core::{
	app_paths,
	game::DbGame,
	game_providers::{
		game_provider::{
			self,
			GameProviderId,
		},
		manual_provider,
		provider_command::ProviderCommandAction,
		steam::{
			steam_provider::Steam,
			steam_shortcut,
		},
	},
	games_query::GamesQuery,
	http::DownloadStatus,
	local_database::{
		game_database::{
			GameDatabase,
			GameIdsResponse,
			attach_remote,
		},
		mod_database::{
			GameModInfo,
			ModDatabase,
		},
	},
	maps::TryGettable,
	mod_providers::mod_provider,
	mods::game_mod::GameMod,
	path_extensions::PathExt,
	remote_config::RemoteConfigs,
	remote_game::{
		self,
	},
	result::LogErrExt,
	user::{
		discord_oauth::{
			DiscordAuthState,
			get_discord_auth_state,
			logout_discord,
			refresh_discord_token_if_possible,
			start_discord_oauth,
		},
		user_socket::start_user_socket_manager,
	},
};
use strum::IntoEnumIterator;
use tauri::{
	AppHandle,
	Manager,
	ipc::Channel,
};
use tauri_plugin_log::{
	Target,
	TargetKind,
};
use tauri_plugin_window_state::StateFlags;
use tauri_specta::Builder;

use crate::result::Result;

mod app_settings;
mod app_state;
mod events;
mod result;
#[cfg(debug_assertions)]
mod typescript;

const DISCORD_TOKEN_REFRESH_INTERVAL: Duration = Duration::from_hours(1);

#[tauri::command]
#[specta::specta]
async fn log_in() -> Result {
	start_discord_oauth().await.map_err(Into::into)
}

#[tauri::command]
#[specta::specta]
async fn get_auth_state() -> Result<DiscordAuthState> {
	get_discord_auth_state().await.map_err(Into::into)
}

#[tauri::command]
#[specta::specta]
async fn log_out() -> Result {
	logout_discord().map_err(Into::into)
}

#[tauri::command]
#[specta::specta]
async fn open_game_folder(
	handle: AppHandle,
	provider_id: GameProviderId,
	game_id: String,
) -> Result {
	handle
		.app_state()
		.database
		.get_game(&provider_id, &game_id)?
		.open_game_folder()?;
	Ok(())
}

#[tauri::command]
#[specta::specta]
async fn open_game_wine_prefix_folder(
	handle: AppHandle,
	provider_id: GameProviderId,
	game_id: String,
) -> Result {
	game_provider::get_provider(provider_id)?
		.get_wine_prefix_path(
			&handle
				.app_state()
				.database
				.get_game(&provider_id, &game_id)?,
		)?
		.open_folder_or_parent()?;

	Ok(())
}

#[tauri::command]
#[specta::specta]
async fn open_game_wine_binary_folder(
	handle: AppHandle,
	provider_id: GameProviderId,
	game_id: String,
) -> Result {
	game_provider::get_provider(provider_id)?
		.get_wine_binary_path(
			&handle
				.app_state()
				.database
				.get_game(&provider_id, &game_id)?,
		)?
		.open_folder_or_parent()?;

	Ok(())
}

#[tauri::command]
#[specta::specta]
async fn open_game_mods_folder(
	handle: AppHandle,
	provider_id: GameProviderId,
	game_id: String,
) -> Result {
	handle
		.app_state()
		.database
		.get_game(&provider_id, &game_id)?
		.open_mods_folder()?;
	Ok(())
}

#[tauri::command]
#[specta::specta]
async fn install_mod(
	provider_id: GameProviderId,
	game_id: String,
	mod_id: &str,
	handle: AppHandle,
) -> Result {
	let state = handle.app_state();
	let game = state.database.get_game(&provider_id, &game_id)?;
	let game_mod = state.database.get_mod(mod_id)?;

	let download_status_channel = state.download_status_channel.read_state()?.clone();

	install_mod_dependencies(&handle, &game_mod, &game).await?;

	if let Some(installed_mod) = state
		.database
		.get_installed_mod(&provider_id, &game_id, mod_id)?
	{
		installed_mod.uninstall()?;
	}

	game_mod
		.install(&game, |status| {
			download_status_channel
				.send(status)
				.ok_or_log("Failed to send download status update");
		})
		.await?;

	state.database.refresh_installed_mods()?;

	handle.emit_safe(events::RefreshGame(provider_id, game_id));

	Ok(())
}

#[tauri::command]
#[specta::specta]
async fn run_mod(
	provider_id: GameProviderId,
	game_id: String,
	mod_id: &str,
	handle: AppHandle,
) -> Result {
	let state = handle.app_state();
	let game = state.database.get_game(&provider_id, &game_id)?;
	state.database.get_mod(mod_id)?.run(&game)?;

	Ok(())
}

#[tauri::command]
#[specta::specta]
async fn configure_mod(
	provider_id: GameProviderId,
	game_id: String,
	mod_id: &str,
	open_folder: bool,
	handle: AppHandle,
) -> Result {
	handle
		.app_state()
		.database
		.try_get_installed_mod(&provider_id, &game_id, mod_id)?
		.configure(open_folder)?;

	Ok(())
}

#[tauri::command]
#[specta::specta]
async fn open_installed_mod_folder(
	provider_id: GameProviderId,
	game_id: String,
	mod_id: &str,
	handle: AppHandle,
) -> Result {
	let state = handle.app_state();
	state
		.database
		.try_get_installed_mod(&provider_id, &game_id, mod_id)?
		.open_folder()?;

	Ok(())
}

#[tauri::command]
#[specta::specta]
async fn refresh_game(handle: AppHandle, provider_id: GameProviderId, game_id: String) -> Result {
	let state = handle.app_state();
	let mut game = state.database.get_game(&provider_id, &game_id)?;
	game.refresh_executable()?;
	state.database.insert_game(&game);
	state.database.refresh_installed_mods()?;

	handle.emit_safe(events::RefreshGame(provider_id, game_id));

	Ok(())
}

#[tauri::command]
#[specta::specta]
async fn uninstall_mod(
	provider_id: GameProviderId,
	game_id: String,
	mod_id: &str,
	handle: AppHandle,
) -> Result {
	let state = handle.app_state();
	state
		.database
		.try_get_installed_mod(&provider_id, &game_id, mod_id)?
		.uninstall()?;
	state.database.refresh_installed_mods()?;

	handle.emit_safe(events::RefreshGame(provider_id, game_id));

	Ok(())
}

#[tauri::command]
#[specta::specta]
async fn uninstall_all_mods(
	handle: AppHandle,
	provider_id: GameProviderId,
	game_id: String,
) -> Result {
	let state = handle.app_state();
	state
		.database
		.get_game(&provider_id, &game_id)?
		.uninstall_all_mods()?;
	state.database.refresh_installed_mods()?;

	handle.emit_safe(events::RefreshGame(provider_id, game_id));

	Ok(())
}

async fn install_mod_dependencies(handle: &AppHandle, game_mod: &GameMod, game: &DbGame) -> Result {
	let state = handle.app_state();

	let relevant_mods = state
		.database
		.get_game_mods(&game.provider_id, &game.game_id)?;
	let download_status_channel = state.download_status_channel.read_state()?.clone();

	if let Some(dependencies) = game_mod.dependencies.as_ref() {
		for dependency in dependencies {
			if let Some(relevant_dependency_mod_info) = relevant_mods.iter().find(|relevant_mod| {
				relevant_mod.compatible && relevant_mod.mod_id == dependency.mod_id
			}) {
				let dependency_mod = state
					.database
					.get_mod(&relevant_dependency_mod_info.mod_id)?;

				Box::pin(install_mod_dependencies(handle, &dependency_mod, game)).await?;

				let outdated = relevant_dependency_mod_info.installed_version.is_none()
					|| relevant_dependency_mod_info.is_outdated;

				if outdated && dependency_mod.install.is_some() {
					dependency_mod
						.install(game, |status| {
							download_status_channel
								.send(status)
								.ok_or_log("Failed to send download status update");
						})
						.await?;
				}
			}
		}
	}

	Ok(())
}

#[tauri::command]
#[specta::specta]
async fn refresh_mods(handle: AppHandle) -> Result {
	let state = handle.app_state();

	mod_provider::refresh_all_mods(&state.database).await?;
	let mods = state.database.get_mod_map()?;
	handle.emit_safe(events::SyncMods(mods));

	Ok(())
}

#[tauri::command]
#[specta::specta]
async fn refresh_remote_games() -> Result {
	let path = remote_game::download_database().await?;
	attach_remote(&path)?;

	Ok(())
}

#[tauri::command]
#[specta::specta]
async fn refresh_games(handle: AppHandle, provider_id: GameProviderId) -> Result {
	let state = handle.app_state();
	provider_id.insert_games(&state.database)?;
	state.database.refresh_installed_mods()?;

	Ok(())
}

#[tauri::command]
#[specta::specta]
async fn add_game(handle: AppHandle, path: PathBuf) -> Result {
	let normalized_path = path.normalize();
	let game = manual_provider::add_game(&normalized_path)?;
	let state = handle.app_state();

	state.database.insert_game(&game);

	handle.emit_safe(events::RefreshGame(game.provider_id, game.game_id.clone()));
	handle.emit_safe(events::AppDatabaseChanged());
	handle.emit_safe(events::SelectGame(GameProviderId::Manual, game.game_id));

	Ok(())
}

#[tauri::command]
#[specta::specta]
async fn remove_game(handle: AppHandle, provider_id: GameProviderId, game_id: String) -> Result {
	let game = handle
		.app_state()
		.database
		.get_game(&provider_id, &game_id)?;

	manual_provider::remove_game(&game)?;

	Ok(())
}

#[tauri::command]
#[specta::specta]
async fn run_provider_command(
	handle: AppHandle,
	provider_id: GameProviderId,
	game_id: &str,
	provider_command_aciton: ProviderCommandAction,
) -> Result {
	let game = handle
		.app_state()
		.database
		.get_game(&provider_id, game_id)?;

	let provider_command = game.provider_commands.try_get(&provider_command_aciton)?;
	provider_command.run(&game)?;

	handle.emit_safe(events::ExecutedProviderCommand);

	Ok(())
}

#[tauri::command]
#[specta::specta]
async fn reset_steam_cache(handle: AppHandle) -> Result {
	Steam::delete_cache()?;

	refresh_games(handle, GameProviderId::Steam).await?;

	Ok(())
}

#[tauri::command]
#[specta::specta]
async fn add_rai_pal_steam_shortcut() -> Result {
	let current_executable = std::env::current_exe()?;
	steam_shortcut::add_current_executable_to_steam_shortcuts(&current_executable)?;

	Ok(())
}

#[tauri::command]
#[specta::specta]
async fn frontend_ready() -> Result {
	// TODO analytics
	// analytics::send_event(analytics::Event::StartApp, "").await;

	Ok(())
}

#[tauri::command]
#[specta::specta]
async fn open_local_mods_folder() -> Result {
	app_paths::local_mods_path()?.open_folder_or_parent()?;

	Ok(())
}

#[tauri::command]
#[specta::specta]
async fn open_logs_folder() -> Result {
	app_paths::logs_path()?.open_folder_or_parent()?;

	Ok(())
}

#[tauri::command]
#[specta::specta]
async fn get_game_ids(handle: AppHandle, query: Option<GamesQuery>) -> Result<GameIdsResponse> {
	let state = handle.app_state();
	Ok(state.database.get_game_ids(query)?)
}

#[tauri::command]
#[specta::specta]
async fn get_game(
	handle: AppHandle,
	provider_id: GameProviderId,
	game_id: String,
) -> Result<DbGame> {
	let state = handle.app_state();
	Ok(state.database.get_game(&provider_id, &game_id)?)
}

#[tauri::command]
#[specta::specta]
async fn get_app_settings() -> Result<AppSettings> {
	Ok(AppSettings::read())
}

#[tauri::command]
#[specta::specta]
async fn save_app_settings(settings: AppSettings) -> Result {
	settings.try_write()
}

#[tauri::command]
#[specta::specta]
async fn get_game_mods(
	provider_id: GameProviderId,
	game_id: String,
	app_handle: AppHandle,
) -> Result<Vec<GameModInfo>> {
	let state = app_handle.app_state();

	Ok(state.database.get_game_mods(&provider_id, &game_id)?)
}

#[tauri::command]
#[specta::specta]
async fn get_remote_configs(
	provider_id: GameProviderId,
	game_id: String,
	app_handle: AppHandle,
) -> Result<Option<RemoteConfigs>> {
	Ok(app_handle
		.app_state()
		.database
		.get_game(&provider_id, &game_id)?
		.get_remote_configs()
		.await?)
}

#[tauri::command]
#[specta::specta]
async fn download_remote_config(
	provider_id: GameProviderId,
	game_id: &str,
	mod_id: &str,
	remote_config_file: &str,
	overwrite: bool,
	app_handle: AppHandle,
) -> Result {
	let state = app_handle.app_state();
	let game = state.database.get_game(&provider_id, game_id)?;
	let game_mod = state.database.get_mod(mod_id)?;

	if let Some(mod_config) = game_mod.config.as_ref() {
		mod_config
			.download(&game, &game_mod, remote_config_file, overwrite)
			.await?;
	}

	Ok(())
}

#[tauri::command]
#[specta::specta]
async fn set_up_global_wine_overrides() -> Result {
	#[cfg(not(target_os = "linux"))]
	{
		use crate::result::Error;

		return Err(Error::LinuxOnly());
	}

	#[cfg(target_os = "linux")]
	{
		use rai_pal_core::wine;

		wine::set_up_global_wine_overrides()?;

		Ok(())
	}
}

#[tauri::command]
#[specta::specta]
async fn listen_to_download_progress(
	handle: AppHandle,
	channel: Channel<DownloadStatus>,
) -> Result {
	handle
		.app_state()
		.download_status_channel
		.write_state_value(channel)?;

	Ok(())
}

fn show_panic(error: &str) {
	rfd::MessageDialog::new()
		.set_title("Rai Pal is panicking rn!")
		.set_description(error)
		.set_buttons(rfd::MessageButtons::Ok)
		.show();
}

fn main() {
	// Since I'm making all exposed functions async, panics won't crash anything important, I think.
	// So I can just catch panics here and show a system message with the error.
	std::panic::set_hook(Box::new(|info| {
		println!("Panic: {info}");

		show_panic(&info.to_string());
	}));

	let builder = Builder::<tauri::Wry>::new()
		.commands(tauri_specta::collect_commands![
			add_game,
			add_rai_pal_steam_shortcut,
			configure_mod,
			get_auth_state,
			log_in,
			log_out,
			frontend_ready,
			get_app_settings,
			get_game_ids,
			get_game,
			get_game_mods,
			install_mod,
			run_mod,
			open_game_folder,
			open_game_wine_prefix_folder,
			open_game_wine_binary_folder,
			open_game_mods_folder,
			open_installed_mod_folder,
			open_local_mods_folder,
			open_logs_folder,
			refresh_game,
			refresh_games,
			refresh_mods,
			refresh_remote_games,
			remove_game,
			reset_steam_cache,
			run_provider_command,
			save_app_settings,
			uninstall_all_mods,
			uninstall_mod,
			get_remote_configs,
			download_remote_config,
			set_up_global_wine_overrides,
			listen_to_download_progress,
		])
		.events(events::collect_events())
		.constant("PROVIDER_IDS", GameProviderId::iter().collect::<Vec<_>>())
		.error_handling(tauri_specta::ErrorHandlingMode::Throw);

	#[cfg(debug_assertions)]
	typescript::export(&builder);

	#[cfg(target_os = "linux")]
	unsafe {
		// This is to fix this error:
		// Error 71 (Protocol error) dispatching to Wayland display.
		// Probably only needed for dev.
		std::env::set_var("__NV_DISABLE_EXPLICIT_SYNC", "1");
	}

	let app_state = AppState::new().unwrap_or_else(|error| {
		show_panic(&format!("Failed to initialize app state. Rai Pal can't work without that, so I'm gonna crash now. Error: {error}"));
		std::process::exit(1);
	});

	tauri::Builder::default()
		.plugin(tauri_plugin_shell::init())
		.plugin(tauri_plugin_os::init())
		.plugin(tauri_plugin_store::Builder::new().build())
		.plugin(
			tauri_plugin_window_state::Builder::default()
				.with_state_flags(StateFlags::POSITION | StateFlags::SIZE)
				.build(),
		)
		.plugin(tauri_plugin_dialog::init())
		.plugin(tauri_plugin_updater::Builder::default().build())
		.plugin(
			tauri_plugin_log::Builder::new()
				.level(log::LevelFilter::Info)
				.targets([
					Target::new(TargetKind::Stdout),
					Target::new(app_paths::logs_path().map_or(
						TargetKind::LogDir { file_name: None },
						|logs_path| TargetKind::Folder {
							path: logs_path,
							file_name: None,
						},
					)),
				])
				.build(),
		)
		.manage(app_state)
		.invoke_handler(builder.invoke_handler())
		.setup(move |app| {
			builder.mount_events(app);

			start_user_socket_manager();

			thread::spawn(|| {
				loop {
					let refresh_result =
						tauri::async_runtime::block_on(refresh_discord_token_if_possible());

					match refresh_result {
						Ok(true) => log::info!("Discord OAuth token auto-refreshed."),
						Ok(false) => {
							log::debug!(
								"Discord OAuth auto-refresh skipped (token not available)."
							);
						}
						Err(error) => {
							log::error!("Failed to auto-refresh Discord OAuth token: {error}");
						}
					}

					thread::sleep(DISCORD_TOKEN_REFRESH_INTERVAL);
				}
			});

			if let Some(window) = app.get_webview_window("main") {
				let mut title = format!("Rai Pal {}", env!("CARGO_PKG_VERSION"));
				if cfg!(debug_assertions) {
					title += " DEV";
				}
				window.set_title(&title)?;

				// Window is created hidden in tauri.conf.json.
				// We show it here once everything is ready, which reduces the jumping around
				// that happens while waiting for tauri_plugin_window_state to do its thing.
				// We could also trigger this on the frontend to reduce the white flash,
				// but it never seems to go away, and that introduces an extra delay
				// until something is visible, so I figure I'd just show it here.
				window.show()?;

				window.on_window_event(|event| {
					if matches!(event, tauri::WindowEvent::Destroyed) {
						// Once the window is closed, we don't need to report panics anymore.
						// I'm doing this because closing the window abruptly while events are being sent
						// causes panics, so it was easy to trigger those messages by just closing while loading data.
						let _ = std::panic::take_hook();
					}
				});
			}

			let app_handle = app.app_handle().clone();

			tauri::async_runtime::spawn(async move {
				let state = app_handle.app_state();
				let cloned_handle = app_handle.clone();

				if let Err(error) = state
					.database
					.lock_db()
					.map_err(|e| e.to_string())
					.and_then(|db| {
						db.update_hook(Some(move |_, _: &str, _: &str, _| {
							cloned_handle.emit_safe(events::AppDatabaseChanged());
						}))
						.map_err(|e| e.to_string())
					}) {
					log::error!(
						"Failed to subscribe to local database updates. App won't work properly. Error: {error}"
					);
				}
			});

			Ok(())
		})
		.run(tauri::generate_context!())
		.unwrap_or_else(|error| {
			#[cfg(target_os = "windows")]
			if let tauri::Error::Runtime(tauri_runtime::Error::CreateWebview(webview_error)) = error
			{
				windows::webview_error_dialog(&webview_error.to_string());
				return;
			}
			#[cfg(target_os = "windows")]
			windows::error_dialog(&error.to_string());

			#[cfg(target_os = "linux")]
			log::error!("Error: {error}");
			#[cfg(target_os = "macos")]
			log::error!("Error: {error}");
		});
}
