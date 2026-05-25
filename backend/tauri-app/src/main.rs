// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
// Command stuff needs to be async so I can spawn tasks.
#![allow(clippy::unused_async)]

use std::{
	collections::HashMap,
	path::PathBuf,
	thread,
	time::{
		Duration,
		SystemTime,
		UNIX_EPOCH,
	},
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
	analytics,
	app_paths,
	game::DbGame,
	game_mods::game_mod::GameMod,
	games_query::GamesQuery,
	http::DownloadStatus,
	local_database::{
		GameDatabase,
		GameIdsResponse,
		attach_remote_database,
	},
	maps::TryGettable,
	path_extensions::PathExt,
	providers::{
		manual_provider,
		provider::{
			self,
			ProviderId,
		},
		provider_command::ProviderCommand,
		steam::{
			steam_provider::Steam,
			steam_shortcut,
		},
	},
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
async fn open_game_folder(handle: AppHandle, provider_id: ProviderId, game_id: String) -> Result {
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
	provider_id: ProviderId,
	game_id: String,
) -> Result {
	provider::get_provider(provider_id)?
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
	provider_id: ProviderId,
	game_id: String,
) -> Result {
	provider::get_provider(provider_id)?
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
	provider_id: ProviderId,
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
async fn open_mods_folder() -> Result {
	app_paths::local_mods_path()?.open_folder_or_parent()?;
	Ok(())
}

#[tauri::command]
#[specta::specta]
async fn open_mod_folder(handle: AppHandle, mod_id: &str) -> Result {
	Ok(handle
		.app_state()
		.local_mods
		.read_state()?
		.try_get(mod_id)?
		.open_local_folder()?)
}

#[tauri::command]
#[specta::specta]
async fn download_mod(handle: AppHandle, mod_id: &str) -> Result {
	download_mod_inner(&handle, mod_id).await?;
	refresh_local_mods(&handle)?;

	Ok(())
}

#[tauri::command]
#[specta::specta]
async fn delete_mod(handle: AppHandle, mod_id: &str) -> Result {
	let state = handle.app_state();
	let local_mods = state.local_mods.read_state()?;
	let local_mod = local_mods.try_get(mod_id)?;

	local_mod.delete_local()?;

	refresh_local_mods(&handle)?;

	Ok(())
}

#[tauri::command]
#[specta::specta]
async fn install_mod(
	provider_id: ProviderId,
	game_id: String,
	mod_id: &str,
	handle: AppHandle,
) -> Result {
	log::info!(
		"Installing mod with id '{mod_id}' for game '{game_id}' from provider '{provider_id}'"
	);

	let state = handle.app_state();
	let game = state.database.get_game(&provider_id, &game_id)?;

	let local_mod = refresh_and_get_local_mod(mod_id, &handle).await?;

	if let Some(dependencies) = local_mod.dependencies.as_ref() {
		for dependency in dependencies {
			let dependency_mod = refresh_and_get_local_mod(&dependency.mod_id, &handle).await?;

			if dependency_mod.install.is_some() {
				dependency_mod.install(&game)?;
			}
		}
	}

	if let Some(installed_mod) = game.get_installed_mod(mod_id)? {
		// Uninstall mod if it already exists, in case there are conflicting leftover files when updating.
		installed_mod.uninstall()?;
	}

	local_mod.install(&game)?;

	handle.emit_safe(events::RefreshGame(provider_id, game_id));

	analytics::send_event(analytics::Event::InstallOrRunMod, mod_id).await;

	Ok(())
}

#[tauri::command]
#[specta::specta]
async fn run_mod(
	provider_id: ProviderId,
	game_id: String,
	mod_id: &str,
	handle: AppHandle,
) -> Result {
	log::info!("Running mod with id '{mod_id}' for game '{game_id}' from provider '{provider_id}'");

	let state = handle.app_state();
	let game = state.database.get_game(&provider_id, &game_id)?;

	let local_mod = refresh_and_get_local_mod(mod_id, &handle).await?;

	local_mod.run(&game)?;

	analytics::send_event(analytics::Event::InstallOrRunMod, mod_id).await;

	Ok(())
}

#[tauri::command]
#[specta::specta]
async fn run_runnable_without_game(handle: AppHandle, mod_id: &str) -> Result {
	let local_mod = refresh_and_get_local_mod(mod_id, &handle).await?;

	local_mod.run_without_game().await?;

	analytics::send_event(analytics::Event::InstallOrRunMod, mod_id).await;

	Ok(())
}

#[tauri::command]
#[specta::specta]
async fn configure_mod(
	provider_id: ProviderId,
	game_id: String,
	mod_id: &str,
	open_folder: bool,
	handle: AppHandle,
) -> Result {
	handle
		.app_state()
		.database
		.get_game(&provider_id, &game_id)?
		.try_get_installed_mod(mod_id)?
		.configure(open_folder)?;

	Ok(())
}

#[tauri::command]
#[specta::specta]
async fn open_installed_mod_folder(
	provider_id: ProviderId,
	game_id: String,
	mod_id: &str,
	handle: AppHandle,
) -> Result {
	let state = handle.app_state();
	let game = state.database.get_game(&provider_id, &game_id)?;
	game.try_get_installed_mod(mod_id)?.open_folder()?;

	Ok(())
}

#[tauri::command]
#[specta::specta]
async fn refresh_game(handle: AppHandle, provider_id: ProviderId, game_id: String) -> Result {
	let state = handle.app_state();
	let mut game = state.database.get_game(&provider_id, &game_id)?;
	game.refresh_executable()?;
	state.database.insert_game(&game);

	handle.emit_safe(events::RefreshGame(provider_id, game_id));

	Ok(())
}

#[tauri::command]
#[specta::specta]
async fn uninstall_mod(
	provider_id: ProviderId,
	game_id: String,
	mod_id: &str,
	handle: AppHandle,
) -> Result {
	let state = handle.app_state();
	let game = state.database.get_game(&provider_id, &game_id)?;
	game.try_get_installed_mod(mod_id)?.uninstall()?;

	handle.emit_safe(events::RefreshGame(provider_id, game_id));

	Ok(())
}

#[tauri::command]
#[specta::specta]
async fn uninstall_all_mods(handle: AppHandle, provider_id: ProviderId, game_id: String) -> Result {
	handle
		.app_state()
		.database
		.get_game(&provider_id, &game_id)?
		.uninstall_all_mods()?;

	handle.emit_safe(events::RefreshGame(provider_id, game_id));

	Ok(())
}

fn refresh_local_mods(handle: &AppHandle) -> Result<HashMap<String, GameMod>> {
	let local_mods = GameMod::get_all_local()?;

	log::info!("Found {} local mods.", { local_mods.len() });
	handle.emit_safe(events::SyncLocalMods(local_mods.clone()));

	handle
		.app_state()
		.local_mods
		.write_state_value(local_mods.clone())?;

	Ok(local_mods)
}

async fn refresh_remote_mods(handle: &AppHandle) -> Result<HashMap<String, GameMod>> {
	let remote_mods = GameMod::get_all_remote(|error| {
		handle.emit_error(format!("Failed to get remote mods: {error}"));
	})
	.await;

	handle.emit_safe(events::SyncRemoteMods(remote_mods.clone()));

	handle
		.app_state()
		.remote_mods
		.write_state_value(remote_mods.clone())?;

	Ok(remote_mods)
}

async fn download_mod_inner(handle: &AppHandle, mod_id: &str) -> Result {
	let state = handle.app_state();
	let remote_mods = state.remote_mods.read_state()?.clone();
	let remote_mod = remote_mods.try_get(mod_id)?;
	let download_status_channel = state.download_status_channel.read_state()?.clone();

	GameMod::download(remote_mod, |status| {
		download_status_channel
			.send(status)
			.ok_or_log("Failed to send download status update");
	})
	.await?;

	Ok(())
}

async fn refresh_and_get_local_mod(mod_id: &str, handle: &AppHandle) -> Result<GameMod> {
	let local_mods = {
		let state = handle.app_state();

		let state_local_mods = state.local_mods.read_state()?.clone();
		if state_local_mods.contains_key(mod_id) {
			Ok(state_local_mods)
		} else {
			// Local mod wasn't in app state,
			// so let's sync app state to local files in case some file was manually changed.
			let disk_local_mods = refresh_local_mods(handle);

			if state_local_mods.contains_key(mod_id) {
				disk_local_mods
			} else {
				download_mod_inner(handle, mod_id).await?;
				refresh_local_mods(handle)
			}
		}
	}?;

	Ok(local_mods.try_get(mod_id).cloned()?)
}

#[tauri::command]
#[specta::specta]
async fn refresh_mods(handle: AppHandle) -> Result {
	refresh_local_mods(&handle)?;
	refresh_remote_mods(&handle).await?;
	refresh_local_mods(&handle)?;

	Ok(())
}

#[tauri::command]
#[specta::specta]
async fn refresh_remote_games(handle: AppHandle) -> Result {
	let state = handle.app_state();
	let path = remote_game::download_database().await?;
	attach_remote_database(state.database.lock_db()?, &path)?;

	Ok(())
}

#[tauri::command]
#[specta::specta]
async fn refresh_games(handle: AppHandle, provider_id: ProviderId) -> Result {
	let state = handle.app_state();

	let start_time = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();

	provider::get_provider(provider_id)?.insert_games(&state.database)?;

	state
		.database
		.remove_stale_games(&provider_id, start_time)?;

	Ok(())
}

#[tauri::command]
#[specta::specta]
async fn add_game(handle: AppHandle, path: PathBuf) -> Result {
	let normalized_path = path.normalize();

	let game = manual_provider::add_game(&normalized_path)?;
	let game_name = game.display_title.clone();

	let state = handle.app_state();

	state.database.insert_game(&game);

	handle.emit_safe(events::RefreshGame(game.provider_id, game.game_id.clone()));

	handle.emit_safe(events::GamesChanged());

	handle.emit_safe(events::SelectGame(ProviderId::Manual, game.game_id));

	analytics::send_event(analytics::Event::ManuallyAddGame, &game_name).await;

	Ok(())
}

#[tauri::command]
#[specta::specta]
async fn remove_game(handle: AppHandle, provider_id: ProviderId, game_id: String) -> Result {
	let game = handle
		.app_state()
		.database
		.get_game(&provider_id, &game_id)?;

	manual_provider::remove_game(&game)?;

	Ok(())
}

#[tauri::command]
#[specta::specta]
async fn run_provider_command(handle: AppHandle, provider_command: ProviderCommand) -> Result {
	provider_command.run()?;

	handle.emit_safe(events::ExecutedProviderCommand);

	Ok(())
}

#[tauri::command]
#[specta::specta]
async fn reset_steam_cache(handle: AppHandle) -> Result {
	Steam::delete_cache()?;

	refresh_games(handle, ProviderId::Steam).await?;

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
	analytics::send_event(analytics::Event::StartApp, "").await;

	Ok(())
}

#[tauri::command]
#[specta::specta]
async fn open_logs_folder() -> Result {
	app_paths::open_logs_folder()?;

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
async fn get_game(handle: AppHandle, provider_id: ProviderId, game_id: String) -> Result<DbGame> {
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
async fn get_installed_mods(
	provider_id: ProviderId,
	game_id: String,
	app_handle: AppHandle,
) -> Result<HashMap<String, GameMod>> {
	let state = app_handle.app_state();
	Ok(state
		.database
		.get_game(&provider_id, &game_id)?
		.get_installed_mods(&state.local_mods.read_state()?))
}

#[tauri::command]
#[specta::specta]
async fn get_remote_configs(
	provider_id: ProviderId,
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
	provider_id: ProviderId,
	game_id: &str,
	mod_id: &str,
	remote_config_file: &str,
	overwrite: bool,
	app_handle: AppHandle,
) -> Result {
	let state = app_handle.app_state();
	let game = state.database.get_game(&provider_id, game_id)?;
	let remote_mods = state.remote_mods.read_state()?.clone();
	let remote_mod = remote_mods.try_get(mod_id)?;
	let local_mods = state.local_mods.read_state()?.clone();
	let local_mod = local_mods.try_get(mod_id)?;

	if let Some(mod_config) = remote_mod.config.as_ref() {
		mod_config
			.download(&game, remote_config_file, overwrite)
			.await?;
		local_mod.update_installed_mod_manifest(&game)?;
	}

	refresh_local_mods(&app_handle)?;

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
			delete_mod,
			download_mod,
			frontend_ready,
			get_app_settings,
			get_game_ids,
			get_game,
			get_installed_mods,
			install_mod,
			run_mod,
			open_game_folder,
			open_game_wine_prefix_folder,
			open_game_wine_binary_folder,
			open_game_mods_folder,
			open_installed_mod_folder,
			open_logs_folder,
			open_mod_folder,
			open_mods_folder,
			refresh_game,
			refresh_games,
			refresh_mods,
			refresh_remote_games,
			remove_game,
			reset_steam_cache,
			run_provider_command,
			run_runnable_without_game,
			save_app_settings,
			uninstall_all_mods,
			uninstall_mod,
			get_remote_configs,
			download_remote_config,
			set_up_global_wine_overrides,
			listen_to_download_progress,
		])
		.events(events::collect_events())
		.constant("PROVIDER_IDS", ProviderId::iter().collect::<Vec<_>>())
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
							cloned_handle.emit_safe(events::GamesChanged());
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
