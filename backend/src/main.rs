// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![deny(clippy::all)]

use anyhow::anyhow;
use std::result::Result as StdResult;
use std::sync::Mutex;
use std::{backtrace::Backtrace, panic};

use game::GameMap;
use serde::Serialize;
use specta::collect_types;
use steam_game::get_steam_games;
use steam_owned_unity_games::{get_steam_owned_unity_games, OwnedUnityGame};
use tauri_specta::ts;

mod appinfo;
mod game;
mod game_executable;
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

#[tauri::command]
#[specta::specta]
async fn get_game_map(state: tauri::State<'_, AppState>) -> CommandResult<GameMap> {
    if let Ok(mutex_guard) = state.game_map.lock() {
        if let Some(game_map) = mutex_guard.clone() {
            return Ok(game_map);
        }
    }

    return match panic::catch_unwind(get_steam_games) {
        Ok(game_map) => {
            if let Ok(mut mutex_guard) = state.game_map.lock() {
                *mutex_guard = Some(game_map.clone());
            }
            Ok(game_map)
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
async fn get_owned_games(state: tauri::State<'_, AppState>) -> CommandResult<Vec<OwnedUnityGame>> {
    if let Ok(mutex_guard) = state.owned_games.lock() {
        if let Some(owned_games) = mutex_guard.clone() {
            return Ok(owned_games);
        }
    }

    return match panic::catch_unwind(get_steam_owned_unity_games) {
        Ok(owned_games_future) => {
            let owned_games = owned_games_future.await?;
            if let Ok(mut mutex_guard) = state.owned_games.lock() {
                *mutex_guard = Some(owned_games.clone());
            }
            Ok(owned_games)
        }
        Err(error) => {
            return Err(anyhow!(
                "{}\nBacktrace:\n{}",
                error
                    .downcast::<&str>()
                    .unwrap_or_else(|_| Box::new("Unknown Source of Error")),
                Backtrace::force_capture()
            )
            .into());
        }
    };
}

#[tauri::command]
#[specta::specta]
async fn open_game_folder(
    game_id: u32,
    executable_id: String,
    state: tauri::State<'_, AppState>,
) -> CommandResult {
    let game_map = get_game_map(state).await?;
    println!("open_game_folder {game_id} {executable_id}");

    if let Some(game) = game_map.get(&game_id) {
        println!("game.name {}", game.name);
        if let Some(executable) = game.executables.get(&executable_id) {
            println!("executable.name {}", executable.name);
            if let Some(parent) = executable.full_path.parent() {
                open::that(parent);
            }
            Ok(())
        } else {
            Err(anyhow!(
                "Failed to find executable with id {executable_id} for game with id {game_id}"
            )
            .into())
        }
    } else {
        Err(anyhow!("Failed to find game with id {game_id}").into())
    }
}

fn main() {
    #[cfg(debug_assertions)]
    ts::export(
        collect_types![get_game_map, get_owned_games, open_game_folder],
        "../frontend/api/bindings.ts",
    )
    .unwrap();

    tauri::Builder::default()
        .manage(AppState {
            game_map: Mutex::default(),
            owned_games: Mutex::default(),
        })
        .invoke_handler(tauri::generate_handler![
            get_game_map,
            get_owned_games,
            open_game_folder,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
