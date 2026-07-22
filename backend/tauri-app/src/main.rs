// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
// Command stuff needs to be async so I can spawn tasks.
#![allow(clippy::unused_async)]

use std::{
	collections::{
		BTreeMap,
		HashSet,
	},
	path::PathBuf,
};

use app_settings::AppSettings;
use app_state::{
	AppState,
	StateData,
	StatefulHandle,
};
use events::{
	AddModSource,
	EventEmitter,
	SelectedGameData,
};
#[cfg(target_os = "windows")]
use rai_pal_core::windows;
use rai_pal_core::{
	analytics,
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
	local_database::{
		app_database::{
			AppDatabase,
			DbMutex,
		},
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
	mod_providers::{
		mod_provider,
		url_mod_provider,
	},
	mods::game_mod::GameMod,
	path_extensions::PathExt,
	progress_status::ProgressStatus,
	remote_config::RemoteConfigs,
	remote_game::{
		self,
	},
	result::LogErrExt,
	user::{
		auth::{
			AuthState,
			get_user_auth_state,
			logout_auth,
			start_auth,
		},
		user_socket::start_user_socket_manager,
	},
};
use strum::IntoEnumIterator;
use tauri::{
	AppHandle,
	Manager,
	WebviewUrl,
	WebviewWindowBuilder,
	ipc::Channel,
};
use tauri_plugin_deep_link::DeepLinkExt;
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

#[tauri::command]
#[specta::specta]
async fn log_in(handle: AppHandle) -> Result {
	start_auth().await?;
	focus(&handle);

	Ok(())
}

#[tauri::command]
#[specta::specta]
async fn get_auth_state() -> Result<AuthState> {
	Ok(get_user_auth_state().await?)
}

#[tauri::command]
#[specta::specta]
async fn log_out() -> Result {
	logout_auth().map_err(Into::into)
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
async fn open_game_data_folder(
	handle: AppHandle,
	provider_id: GameProviderId,
	game_id: String,
) -> Result {
	handle
		.app_state()
		.database
		.get_game(&provider_id, &game_id)?
		.open_data_folder()?;
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

	if let Ok(game_mod) = database.get_mod(mod_id)
		&& let Some(deps) = &game_mod.dependencies
	{
		for dep in deps {
			if let Some(info) = relevant_mods
				.iter()
				.find(|m| m.compatible && m.mod_id == dep.mod_id)
			{
				let outdated = info.installed_version.is_none() || info.is_outdated;
				if outdated
					&& let Ok(dep_mod) = database.get_mod(&dep.mod_id)
					&& dep_mod.install.is_some()
				{
					collect_deps_to_install(&dep.mod_id, database, relevant_mods, visited, result);
					result.push(dep_mod);
				}
			}
		}
	}
}

fn build_steps(
	mods: &[GameMod],
	main_id: &str,
) -> (Vec<(String, String)>, Option<String>, Option<String>) {
	let mut steps = Vec::new();
	let mut main_dl = None;
	let mut main_ex = None;
	for m in mods {
		if m.download.is_none() {
			continue;
		}
		let dl_key = format!("{}:download", m.id);
		steps.push((dl_key.clone(), format!("Download {}", m.id)));
		if m.id == main_id {
			main_dl = Some(dl_key);
		}
		if m.install
			.as_ref()
			.and_then(|i| i.extract.as_ref())
			.is_some()
		{
			let ex_key = format!("{}:extract", m.id);
			steps.push((ex_key.clone(), format!("Extract {}", m.id)));
			if m.id == main_id {
				main_ex = Some(ex_key);
			}
		}
	}
	(steps, main_dl, main_ex)
}

#[tauri::command]
#[specta::specta]
async fn install_mod(
	mod_id: &str,
	provider_id_option: Option<GameProviderId>,
	game_id_option: Option<String>,
	handle: AppHandle,
) -> Result {
	let state = handle.app_state();

	let game_option = if let Some(game_id) = game_id_option
		&& let Some(provider_id) = provider_id_option
	{
		Some(state.database.get_game(&provider_id, &game_id)?)
	} else {
		None
	};
	let game_mod = state.database.get_mod(mod_id)?;

	let download_status_channel = state.download_status_channel.read_state()?.clone();

	let relevant_mods: Vec<GameModInfo> = if let Some(game) = game_option.as_ref() {
		state
			.database
			.get_game_mods(&game.provider_id, &game.game_id)?
	} else {
		state
			.database
			.get_mod_map()?
			.values()
			.map(|other_mod| GameModInfo {
				compatible: true,
				has_installed_dependants: false,
				installed_hash: None,
				installed_version: None,
				is_outdated: false,
				mod_id: other_mod.id.clone(),
				mod_scope: other_mod.scope.clone().unwrap_or_default(),
			})
			.collect()
	};

	let mut deps_to_install = Vec::new();
	collect_deps_to_install(
		mod_id,
		&state.database,
		&relevant_mods,
		&mut HashSet::new(),
		&mut deps_to_install,
	);

	let all_owned: Vec<GameMod> = deps_to_install
		.iter()
		.chain(std::iter::once(&game_mod))
		.cloned()
		.collect();

	let (steps, main_dl, _main_ex) = build_steps(&all_owned, mod_id);

	let channel = download_status_channel.clone();
	let forward = move |status: ProgressStatus| {
		channel
			.send(status)
			.ok_or_log("Failed to send download status update");
	};

	for (step_id, step_name) in &steps {
		forward(ProgressStatus::Pending {
			id: step_id.clone(),
			name: step_name.clone(),
		});
	}

	// Prevent concurrent mod installs since it can get messy with shared dependencies.
	// Acquired after sending pending state so the frontend always sees updates immediately.
	let _install_guard = state.install_lock.lock().await;

	let result: std::result::Result<(), rai_pal_core::result::Error> = async {
		for m in &deps_to_install {
			m.install(game_option.as_ref(), &forward).await?;
		}

		if let Some(game) = game_option.as_ref()
			&& let Some(installed_mod) = state.database.get_installed_mod(
				mod_id,
				Some(game.provider_id),
				Some(game.game_id.clone()),
			)? {
			installed_mod.uninstall()?;
		}

		if main_dl.is_some() {
			game_mod.install(game_option.as_ref(), &forward).await?;
		}

		state.database.refresh_installed_mods()?;

		if let Some(game) = game_option.as_ref() {
			handle.emit_safe(events::RefreshGame(game.provider_id, game.game_id.clone()));
		}

		for (step_id, _) in &steps {
			forward(ProgressStatus::Finished {
				id: step_id.clone(),
			});
		}

		Ok(())
	}
	.await;

	if let Err(error) = &result {
		for (step_id, _) in &steps {
			forward(ProgressStatus::Failed {
				id: step_id.clone(),
				error: error.to_string(),
			});
		}
	}

	result?;

	Ok(())
}

#[tauri::command]
#[specta::specta]
async fn run_mod(
	mod_id: &str,
	provider_id_option: Option<GameProviderId>,
	game_id_option: Option<String>,
	handle: AppHandle,
) -> Result {
	let state = handle.app_state();

	let game = if let Some(game_id) = game_id_option
		&& let Some(provider_id) = provider_id_option
	{
		Some(state.database.get_game(&provider_id, &game_id)?)
	} else {
		None
	};

	state.database.get_mod(mod_id)?.run(game.as_ref())?;

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

#[tauri::command]
#[specta::specta]
async fn refresh_mods(handle: AppHandle) -> Result {
	let state = handle.app_state();
	mod_provider::refresh_all_mods(&state.database).await?;

	Ok(())
}

#[tauri::command]
#[specta::specta]
async fn get_url_mod_sources() -> Result<url_mod_provider::UrlModSources> {
	Ok(url_mod_provider::get_url_mod_sources())
}

#[tauri::command]
#[specta::specta]
async fn add_url_mod_source(url: String) -> Result {
	Ok(url_mod_provider::add_url_mod_source(url)?)
}

#[tauri::command]
#[specta::specta]
async fn get_mods_from_url_mod_source(url: String) -> Result<Vec<GameMod>> {
	Ok(url_mod_provider::get_mods_from_url_mod_source(&url).await?)
}

#[tauri::command]
#[specta::specta]
async fn remove_url_mod_source(url: String) -> Result {
	Ok(url_mod_provider::remove_url_mod_source(&url)?)
}

#[tauri::command]
#[specta::specta]
async fn get_mods(handle: AppHandle) -> Result<BTreeMap<String, GameMod>> {
	let state = handle.app_state();
	Ok(state.database.get_mod_map()?)
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

	let mod_infos = state
		.database
		.get_game_mods(&GameProviderId::Manual, &game.game_id)?;
	let data = SelectedGameData { game, mod_infos };
	handle.emit_safe(events::SelectGame(Some(data)));

	Ok(())
}

#[tauri::command]
#[specta::specta]
async fn add_game_directory(handle: AppHandle, path: PathBuf) -> Result {
	let normalized_path = path.normalize();
	let games = manual_provider::add_directory(&normalized_path)?;
	let state = handle.app_state();

	for game in &games {
		state.database.insert_game(game);
		handle.emit_safe(events::RefreshGame(game.provider_id, game.game_id.clone()));
	}

	handle.emit_safe(events::AppDatabaseChanged());

	if let Some(game) = games.first() {
		let mod_infos = state
			.database
			.get_game_mods(&GameProviderId::Manual, &game.game_id)?;
		let data = SelectedGameData {
			game: game.clone(),
			mod_infos,
		};
		handle.emit_safe(events::SelectGame(Some(data)));
	}

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
async fn send_analytics_event(
	event: analytics::Event,
	data: Option<analytics::AnalyticsData>,
) -> Result {
	analytics::send_event(event, data).await;

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
	channel: Channel<ProgressStatus>,
) -> Result {
	handle
		.app_state()
		.download_status_channel
		.write_state_value(channel)?;

	Ok(())
}

#[tauri::command]
#[specta::specta]
async fn set_selected_game(
	provider_id: Option<GameProviderId>,
	game_id: Option<String>,
	handle: AppHandle,
) -> Result {
	let state = handle.app_state();

	if let Some(provider_id) = provider_id
		&& let Some(game_id) = game_id
	{
		let game = state.database.get_game(&provider_id, &game_id)?;
		let mod_infos = state.database.get_game_mods(&provider_id, &game_id)?;

		*state
			.selected_game
			.write()
			.map_err(|e| crate::result::Error::FailedToAccessStateData(e.to_string()))? =
			Some((provider_id, game_id));

		let data = SelectedGameData { game, mod_infos };
		handle.emit_safe(events::SelectGame(Some(data)));
	} else {
		*state
			.selected_game
			.write()
			.map_err(|e| crate::result::Error::FailedToAccessStateData(e.to_string()))? = None;

		handle.emit_safe(events::SelectGame(None));
	}

	Ok(())
}

fn show_panic(error: &str) {
	rfd::MessageDialog::new()
		.set_title("Rai Pal is panicking rn!")
		.set_description(error)
		.set_buttons(rfd::MessageButtons::Ok)
		.show();
}

fn handle_deep_link_url(url: &str, handle: &AppHandle) {
	let url = url
		.strip_prefix("rai-pal://")
		.or_else(|| url.strip_prefix("rai-pal:"));

	let Some(url) = url else { return };

	let (path, query) = url.split_once('?').unwrap_or((url, ""));

	if path != "add-mod-source" {
		return;
	}

	let mod_source_url = query.split('&').find_map(|part| part.strip_prefix("url="));

	if let Some(mod_source_url) = mod_source_url {
		handle.emit_safe(AddModSource(mod_source_url.to_string()));
	}
}

fn focus(handle: &AppHandle) {
	if let Some(window) = handle.get_webview_window("main") {
		window.unminimize().ok_or_log("Failed to unminimize window");
		window.set_focus().ok_or_log("Failed to focus window");
	} else {
		log::error!("Failed to find main window!");
	}
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
			add_game_directory,
			add_rai_pal_steam_shortcut,
			add_url_mod_source,
			configure_mod,
			download_remote_config,
			get_app_settings,
			get_auth_state,
			get_game,
			get_game_ids,
			get_game_mods,
			get_mods,
			get_mods_from_url_mod_source,
			get_remote_configs,
			get_url_mod_sources,
			install_mod,
			listen_to_download_progress,
			log_in,
			log_out,
			open_game_folder,
			open_game_mods_folder,
			open_game_data_folder,
			open_game_wine_binary_folder,
			open_game_wine_prefix_folder,
			open_installed_mod_folder,
			open_local_mods_folder,
			open_logs_folder,
			refresh_game,
			refresh_games,
			refresh_mods,
			refresh_remote_games,
			remove_game,
			remove_url_mod_source,
			reset_steam_cache,
			run_mod,
			set_selected_game,
			run_provider_command,
			save_app_settings,
			send_analytics_event,
			set_up_global_wine_overrides,
			uninstall_all_mods,
			uninstall_mod,
		])
		.events(events::collect_events())
		.constant("PROVIDER_IDS", GameProviderId::iter().collect::<Vec<_>>())
		.error_handling(tauri_specta::ErrorHandlingMode::Throw);

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
		.plugin(tauri_plugin_single_instance::init(|handle, _args, _cwd| {
			focus(handle);
		}))
		.plugin(
			tauri_plugin_log::Builder::new()
				.level(if cfg!(debug_assertions) {
					log::LevelFilter::Trace
				} else {
					log::LevelFilter::Info
				})
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
		.plugin(tauri_plugin_deep_link::init())
		.plugin(tauri_plugin_os::init())
		.plugin(
			tauri_plugin_window_state::Builder::default()
				.with_state_flags(StateFlags::POSITION | StateFlags::SIZE | StateFlags::MAXIMIZED)
				.build(),
		)
		.plugin(tauri_plugin_dialog::init())
		.plugin(tauri_plugin_updater::Builder::default().build())
		.manage(app_state)
		.invoke_handler(builder.invoke_handler())
		.setup(move |app| {
			builder.mount_events(app);

			// --- Deep link ---

			app.deep_link().register_all()?;

			let handle = app.handle().clone();

			if let Ok(Some(urls)) = app.deep_link().get_current() {
				log::info!("App opened via deep link: {urls:?}");

				for url in &urls {
					handle_deep_link_url(url.as_str(), &handle);
				}
			} else {
				log::info!("App opened directly.");

				#[cfg(debug_assertions)]
				// This is buried here to avoid touching the TS bindings file when opening via deep link.
				typescript::export(&builder);
			}

			app.deep_link().on_open_url(move |event| {
				let urls = event.urls();

				log::info!("Deep link received: {urls:?}");

				for url in urls {
					handle_deep_link_url(url.as_str(), &handle);
				}
			});

			// --- Window ---

			// Only create the window once everything is ready, which reduces the jumping around
			// that happens while waiting for tauri_plugin_window_state to do its thing.
			// We could also trigger this on the frontend to reduce the white flash,
			// but it never seems to go away, and that introduces an extra delay
			// until something is visible, so I figure I'd just show it here.
			let window = WebviewWindowBuilder::new(app, "main", WebviewUrl::default())
				.title(format!(
					"Rai Pal {}{}",
					env!("CARGO_PKG_VERSION"),
					if cfg!(debug_assertions) { " DEV" } else { "" }
				))
				// Another reason to create Webview manually is to have full control of the data folder.
				.data_directory(app_paths::app_data_subfolder("main-webview")?)
				.inner_size(800.0, 600.0)
				.min_inner_size(800.0, 500.0)
				.focusable(true)
				.build()?;

			window.on_window_event(|event| {
				if matches!(event, tauri::WindowEvent::Destroyed) {
					// Once the window is closed, we don't need to report panics anymore.
					// I'm doing this because closing the window abruptly while events are being sent
					// causes panics, so it was easy to trigger those messages by just closing while loading data.
					let _ = std::panic::take_hook();
				}
			});

			// --- Background tasks ---

			tauri::async_runtime::spawn(start_user_socket_manager());

			tauri::async_runtime::spawn({
				let app_handle = app.app_handle().clone();
				async move {
					let state = app_handle.app_state();

					if let Err(error) = state
						.database
						.lock_db()
						.map_err(|e| e.to_string())
						.and_then(|db| {
							let db_handle = app_handle.clone();
							db.update_hook(Some(move |_, _: &str, _: &str, _| {
								db_handle.emit_safe(events::AppDatabaseChanged());
							}))
							.map_err(|e| e.to_string())
						}) {
						log::error!(
							"Failed to subscribe to local database updates. App won't work properly. Error: {error}"
						);
					}
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

			// Use eprintln! as fallback since log plugin may not capture this
			eprintln!("Fatal error: {error}");
			log::error!("Fatal error: {error}");
		});
}
