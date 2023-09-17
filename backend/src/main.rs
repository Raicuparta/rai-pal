// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![deny(clippy::all)]

use anyhow::anyhow;
use game::GameMap;
use mod_loader::BepInEx;
use specta::ts::{BigIntExportBehavior, ExportConfiguration};
use std::future::Future;
use std::result::Result as StdResult;
use std::sync::Mutex;
use std::{backtrace::Backtrace, panic};
use tauri::api::dialog::message;

use serde::Serialize;
use specta::collect_types;
use steam_game::get_steam_games;
use steam_owned_unity_games::{get_steam_owned_unity_games, OwnedUnityGame};
use tauri_specta::ts;

mod appinfo;
mod files;
mod game;
mod r#mod;
mod mod_loader;
mod steam_game;
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
    game_map: Mutex<Option<GameMap>>,
    owned_games: Mutex<Option<Vec<OwnedUnityGame>>>,
}

async fn get_state_data<TData, TFunction, TFunctionResult>(
    mutex: &Mutex<Option<TData>>,
    function: TFunction,
    ignore_cache: bool,
) -> CommandResult<TData>
where
    TFunction: Fn() -> TFunctionResult + std::panic::UnwindSafe,
    TData: Clone,
    TFunctionResult: Future<Output = Result<TData>>,
{
    if !ignore_cache {
        if let Ok(mutex_guard) = mutex.lock() {
            if let Some(data) = mutex_guard.clone() {
                return Ok(data);
            }
        }
    }

    return match panic::catch_unwind(function) {
        Ok(result) => {
            let data = result.await?;
            if let Ok(mut mutex_guard) = mutex.lock() {
                *mutex_guard = Some(data.clone());
            }
            Ok(data)
        }
        Err(error) => Err(anyhow!(
            "{}\nBacktrace:\n{}",
            error
                .downcast::<&str>()
                .unwrap_or_else(|_| Box::new("Unknown Source of Error")),
            Backtrace::force_capture()
        )
        .into()),
    };
}

#[tauri::command]
#[specta::specta]
async fn get_game_map(
    state: tauri::State<'_, AppState>,
    ignore_cache: bool,
) -> CommandResult<GameMap> {
    get_state_data(&state.game_map, get_steam_games, ignore_cache).await
}

#[tauri::command]
#[specta::specta]
async fn get_owned_games(
    state: tauri::State<'_, AppState>,
    ignore_cache: bool,
) -> CommandResult<Vec<OwnedUnityGame>> {
    get_state_data(
        &state.owned_games,
        get_steam_owned_unity_games,
        ignore_cache,
    )
    .await
}

#[tauri::command]
#[specta::specta]
async fn get_mod_loaders(handle: tauri::AppHandle) -> CommandResult<Vec<BepInEx>> {
    Ok(Vec::from([BepInEx::new(
        &handle
            .path_resolver()
            .resolve_resource("resources/bepinex")
            .ok_or(anyhow!("Failed to find BepInEx folder"))?,
    )?]))
}

#[tauri::command]
#[specta::specta]
async fn open_game_folder(game_id: String, state: tauri::State<'_, AppState>) -> CommandResult {
    let game_map = get_game_map(state, false).await?;
    let game = game_map
        .get(&game_id)
        .ok_or(anyhow!("Failed to find game with id {game_id}"))?;

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
        .ok_or(anyhow!("Failed to find game with id {game_id}"))?;

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
    let mod_loaders = get_mod_loaders(handle).await?;
    let mod_loader = mod_loaders
        .iter()
        .find(|loader| loader.id == mod_loader_id)
        .ok_or(anyhow!("Failed to find mod loader with id {mod_loader_id}"))?;

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
        .ok_or(anyhow!("Failed to find game with id {game_id}"))?;

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
    let mod_loaders = get_mod_loaders(handle).await?;
    let mod_loader = mod_loaders
        .iter()
        .find(|loader| loader.id == mod_loader_id)
        .ok_or(anyhow!("Failed to find mod loader with id {mod_loader_id}"))?;

    let game_map = get_game_map(state, false).await?;
    let game = game_map
        .get(&game_id)
        .ok_or(anyhow!("Failed to find game with ID {game_id}"))?;

    mod_loader
        .install_mod(game, mod_id)
        .map_err(|err| anyhow!("Failed to install mod: {err}").into())
}

fn main() {
    #[cfg(debug_assertions)]
    ts::export_with_cfg(
        collect_types![
            get_game_map,
            get_owned_games,
            open_game_folder,
            get_mod_loaders,
            install_mod,
            open_game_mods_folder,
            start_game,
            open_mod_folder,
        ]
        .unwrap(),
        ExportConfiguration::default().bigint(BigIntExportBehavior::BigInt),
        "../frontend/api/bindings.ts",
    )
    .unwrap();

    std::panic::set_hook(Box::new(|info| {
        message(
            None::<&tauri::Window>,
            "Failed to execute command",
            info.to_string(),
        );
        println!("Panic: {info}");
    }));

    tauri::Builder::default()
        .manage(AppState {
            game_map: Mutex::default(),
            owned_games: Mutex::default(),
        })
        .invoke_handler(tauri::generate_handler![
            get_game_map,
            get_owned_games,
            open_game_folder,
            get_mod_loaders,
            install_mod,
            open_game_mods_folder,
            start_game,
            open_mod_folder,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
