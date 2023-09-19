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

use anyhow::anyhow;
use mod_loaders::mod_loader::{self, ModLoaderData};
use serde::Serialize;
use specta::ts::{BigIntExportBehavior, ExportConfiguration};
use std::future::Future;
use std::result::Result as StdResult;
use std::sync::Mutex;
use steam_owned_unity_games::OwnedUnityGame;
use tauri::api::dialog::message;

mod appinfo;
mod files;
mod game;
mod game_mod;
mod macros;
mod mod_loaders;
mod steam_games;
mod steam_owned_unity_games;

struct Error(anyhow::Error);

impl From<anyhow::Error> for Error {
    fn from(item: anyhow::Error) -> Self {
        Self(item)
    }
}

type CommandResult<T = ()> = StdResult<T, Error>;
pub type Result<T = ()> = StdResult<T, anyhow::Error>;

impl Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> StdResult<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.0.to_string())
    }
}

struct AppState {
    game_map: Mutex<Option<game::Map>>,
    owned_games: Mutex<Option<Vec<OwnedUnityGame>>>,
}

async fn get_state_data<TData, TFunction, TFunctionResult>(
    mutex: &Mutex<Option<TData>>,
    function: TFunction,
    ignore_cache: bool,
) -> CommandResult<TData>
where
    TFunction: Fn() -> TFunctionResult + std::panic::UnwindSafe + Send,
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

    let result = function();
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
) -> CommandResult<game::Map> {
    get_state_data(&state.game_map, steam_games::get, ignore_cache).await
}

#[tauri::command]
#[specta::specta]
async fn get_owned_games(
    state: tauri::State<'_, AppState>,
    ignore_cache: bool,
) -> CommandResult<Vec<OwnedUnityGame>> {
    get_state_data(
        &state.owned_games,
        steam_owned_unity_games::get,
        ignore_cache,
    )
    .await
}

#[tauri::command]
#[specta::specta]
async fn get_mod_loaders(handle: tauri::AppHandle) -> CommandResult<Vec<ModLoaderData>> {
    Ok(mod_loader::get_all(
        &handle
            .path_resolver()
            .resolve_resource("resources")
            .ok_or_else(|| anyhow!("Failed to find resources path"))?,
    )?
    .iter()
    .map(|loader| loader.get_data())
    .collect())
}

#[tauri::command]
#[specta::specta]
async fn open_game_folder(game_id: String, state: tauri::State<'_, AppState>) -> CommandResult {
    let game_map = get_game_map(state, false).await?;
    let game = game_map
        .get(&game_id)
        .ok_or_else(|| anyhow!("Failed to find game with id {game_id}"))?;

    game.open_game_folder()
        .map_err(|error| anyhow!("Failed to open game folder: {error}").into())
}

#[tauri::command]
#[specta::specta]
async fn open_game_mods_folder(
    game_id: String,
    state: tauri::State<'_, AppState>,
) -> CommandResult {
    let game_map = get_game_map(state, false).await?;
    let game = game_map
        .get(&game_id)
        .ok_or_else(|| anyhow!("Failed to find game with id {game_id}"))?;

    game.open_mods_folder()
        .map_err(|error| anyhow!("Failed to open game folder: {error}").into())
}

#[tauri::command]
#[specta::specta]
async fn open_mod_folder(
    mod_loader_id: String,
    mod_id: String,
    handle: tauri::AppHandle,
) -> CommandResult {
    let mod_loader = mod_loader::find(
        &handle
            .path_resolver()
            .resolve_resource("resources")
            .ok_or_else(|| anyhow!("Failed to find resources folder"))?,
        &mod_loader_id,
    )?;

    mod_loader
        .open_mod_folder(mod_id)
        .map_err(|err| anyhow!("Failed to open mod folder: {err}").into())
}

#[tauri::command]
#[specta::specta]
async fn start_game(game_id: String, state: tauri::State<'_, AppState>) -> CommandResult {
    let game_map = get_game_map(state, false).await?;
    let game = game_map
        .get(&game_id)
        .ok_or_else(|| anyhow!("Failed to find game with id {game_id}"))?;

    game.start()
        .map_err(|error| anyhow!("Failed to open game folder: {error}").into())
}

#[tauri::command]
#[specta::specta]
async fn install_mod(
    mod_loader_id: String,
    mod_id: String,
    game_id: String,
    state: tauri::State<'_, AppState>,
    handle: tauri::AppHandle,
) -> CommandResult {
    let game_map = get_game_map(state, false).await?;
    let game = game_map
        .get(&game_id)
        .ok_or_else(|| anyhow!("Failed to find game with ID {game_id}"))?;

    let mod_loader = mod_loader::find(
        &handle
            .path_resolver()
            .resolve_resource("resources")
            .ok_or_else(|| anyhow!("Failed to find resources folder"))?,
        &mod_loader_id,
    )?;
    mod_loader.install_mod(game, mod_id.clone()).map_err(|err| {
        anyhow!(
            "Failed to install mod '{mod_id}' from mod loader {mod_loader_id} on game {game_id}: {err}",
        )
        .into()
    })
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
