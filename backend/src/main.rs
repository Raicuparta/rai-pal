// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![deny(clippy::nursery)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::as_conversions)]
#![deny(clippy::clone_on_ref_ptr)]
#![deny(clippy::decimal_literal_representation)]
#![deny(clippy::shadow_unrelated)]
#![deny(clippy::verbose_file_reads)]
#![allow(clippy::used_underscore_binding)]
#![allow(clippy::unused_async)]
#![allow(clippy::module_name_repetitions)]

use std::{
	future::Future,
	path::PathBuf,
	result::Result as StdResult,
	sync::Mutex,
};

use mod_loaders::mod_loader::{
	self,
	ModLoaderActions,
};
use specta::ts::{
	BigIntExportBehavior,
	ExportConfiguration,
};
use steam_owned_unity_games::OwnedUnityGame;
use tauri::api::dialog::message;

mod appinfo;
mod files;
mod game;
mod game_mod;
mod macros;
mod mod_loaders;
mod paths;
mod steam_games;
mod steam_owned_unity_games;

#[derive(Debug, thiserror::Error)]
pub enum Error {
	#[error(transparent)]
	Io(#[from] std::io::Error),

	#[error(transparent)]
	Glob(#[from] glob::PatternError),

	#[error(transparent)]
	Reqwest(#[from] reqwest::Error),

	#[error(transparent)]
	Goblin(#[from] goblin::error::Error),

	#[error("Invalid type `{0}` in binary vdf key/value pair")]
	InvalidBinaryVdfType(u8),

	#[error("Failed to get file name from path `{0}`")]
	FailedToGetFileName(PathBuf),

	#[error("Failed to install mod `{0}`")]
	ModInstallFailure(PathBuf),

	#[error("Failed to find game with ID `{0}`")]
	GameNotFound(String),

	#[error("Failed to find mod with ID `{0}`")]
	ModNotFound(String),

	#[error("Failed to find mod loader with ID `{0}`")]
	ModLoaderNotFound(String),

	#[error("Failed to find Steam on this system")]
	SteamNotFound(),

	#[error("Failed to find Rai Pal resources folder")]
	ResourcesNotFound(),

	#[error("Failed to find Rai Pal app data folder")]
	AppDataNotFound(),

	#[error("Failed to parse path (possibly because is a non-UTF-8 string) `{0}`")]
	PathParseFailure(PathBuf),

	#[error("Failed to get folder parent for path `{0}`")]
	PathParentNotFound(PathBuf),

	#[error("Tried to read empty file `{0}`")]
	EmptyFile(PathBuf),
}

impl serde::Serialize for Error {
	fn serialize<S>(&self, serializer: S) -> StdResult<S::Ok, S::Error>
	where
		S: serde::ser::Serializer,
	{
		serializer.serialize_str(self.to_string().as_ref())
	}
}

pub type Result<T = ()> = StdResult<T, Error>;

struct AppState {
	game_map: Mutex<Option<game::Map>>,
	owned_games: Mutex<Option<Vec<OwnedUnityGame>>>,
	mod_loaders: Mutex<Option<mod_loader::DataMap>>,
}

async fn get_state_data<TData, TFunction, TFunctionResult>(
	mutex: &Mutex<Option<TData>>,
	function: TFunction,
	ignore_cache: bool,
	app_handle: tauri::AppHandle,
) -> Result<TData>
where
	TFunction: Fn(tauri::AppHandle) -> TFunctionResult + std::panic::UnwindSafe + Send,
	TData: Clone + Send,
	TFunctionResult: Future<Output = Result<TData>> + Send,
{
	if !ignore_cache {
		if let Ok(mutex_guard) = mutex.lock() {
			if let Some(data) = mutex_guard.clone() {
				return Ok(data);
			}
		}
	}

	let result = function(app_handle);
	let data = result.await?;
	if let Ok(mut mutex_guard) = mutex.lock() {
		*mutex_guard = Some(data.clone());
	}

	Ok(data)
}

#[tauri::command]
#[specta::specta]
async fn get_game_map(
	state: tauri::State<'_, AppState>,
	ignore_cache: bool,
	handle: tauri::AppHandle,
) -> Result<game::Map> {
	get_state_data(&state.game_map, steam_games::get, ignore_cache, handle).await
}

#[tauri::command]
#[specta::specta]
async fn get_owned_games(
	state: tauri::State<'_, AppState>,
	ignore_cache: bool,
	handle: tauri::AppHandle,
) -> Result<Vec<OwnedUnityGame>> {
	get_state_data(
		&state.owned_games,
		steam_owned_unity_games::get,
		ignore_cache,
		handle,
	)
	.await
}

#[tauri::command]
#[specta::specta]
async fn get_mod_loaders(
	state: tauri::State<'_, AppState>,
	ignore_cache: bool,
	handle: tauri::AppHandle,
) -> Result<mod_loader::DataMap> {
	get_state_data(
		&state.mod_loaders,
		mod_loader::get_data_map,
		ignore_cache,
		handle,
	)
	.await
}

#[tauri::command]
#[specta::specta]
async fn open_game_folder(
	game_id: String,
	state: tauri::State<'_, AppState>,
	handle: tauri::AppHandle,
) -> Result {
	let game_map = get_game_map(state, false, handle).await?;
	let game = game_map
		.get(&game_id)
		.ok_or_else(|| Error::GameNotFound(game_id))?;

	game.open_game_folder()
}

#[tauri::command]
#[specta::specta]
async fn open_game_mods_folder(
	game_id: String,
	state: tauri::State<'_, AppState>,
	handle: tauri::AppHandle,
) -> Result {
	let game_map = get_game_map(state, false, handle).await?;
	let game = game_map
		.get(&game_id)
		.ok_or_else(|| Error::GameNotFound(game_id))?;

	game.open_mods_folder()
}

#[tauri::command]
#[specta::specta]
async fn open_mod_folder(
	mod_loader_id: String,
	mod_id: String,
	handle: tauri::AppHandle,
) -> Result {
	let mod_loader = mod_loader::get(&paths::resources_path(&handle)?, &mod_loader_id)?;

	mod_loader.open_mod_folder(mod_id)
}

#[tauri::command]
#[specta::specta]
async fn start_game(
	game_id: String,
	state: tauri::State<'_, AppState>,
	handle: tauri::AppHandle,
) -> Result {
	let game_map = get_game_map(state, false, handle).await?;
	let game = game_map
		.get(&game_id)
		.ok_or_else(|| Error::GameNotFound(game_id))?;

	game.start()
}

#[tauri::command]
#[specta::specta]
async fn install_mod(
	mod_loader_id: String,
	mod_id: String,
	game_id: String,
	state: tauri::State<'_, AppState>,
	handle: tauri::AppHandle,
) -> Result {
	let game_map = get_game_map(state, false, handle.clone()).await?;
	let game = game_map
		.get(&game_id)
		.ok_or_else(|| Error::GameNotFound(game_id))?;

	let mod_loader = mod_loader::get(&paths::resources_path(&handle)?, &mod_loader_id)?;

	mod_loader.install_mod(game, mod_id.clone())
}

fn main() {
	// Since I'm making all exposed functions async, panics won't crash anything important, I think.
	// So I can just catch panics here and show a system message with the error.
	std::panic::set_hook(Box::new(|info| {
		println!("Panic: {info}");
		message(
			None::<&tauri::Window>,
			"Failed to execute command",
			info.to_string(),
		);
	}));

	set_up_tauri!(
		"../frontend/api/bindings.ts",
		AppState {
			game_map: Mutex::default(),
			owned_games: Mutex::default(),
			mod_loaders: Mutex::default(),
		},
		[
			get_game_map,
			get_owned_games,
			open_game_folder,
			get_mod_loaders,
			install_mod,
			open_game_mods_folder,
			start_game,
			open_mod_folder
		]
	);
}
