// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{backtrace::Backtrace, panic};

use game::GameMap;
use specta::collect_types;
use steam_game::get_steam_games;
use steam_owned_unity_games::{get_steam_owned_unity_games, OwnedUnityGame};
use tauri_specta::ts;

mod appinfo;
mod game;
mod game_executable;
mod steam_game;
mod steam_owned_unity_games;

#[tauri::command]
#[specta::specta]
async fn get_game_map() -> Result<GameMap, String> {
    return match panic::catch_unwind(|| get_steam_games()) {
        Ok(game_map) => Ok(game_map),
        Err(error) => {
            return Err(format!(
                "{}\nBacktrace:\n{}",
                error
                    .downcast::<&str>()
                    .unwrap_or(Box::new("Unknown Source of Error")),
                Backtrace::force_capture().to_string()
            ));
        }
    };
}

#[tauri::command]
#[specta::specta]
async fn get_owned_games() -> Result<Vec<OwnedUnityGame>, String> {
    return match panic::catch_unwind(|| get_steam_owned_unity_games()) {
        Ok(game_map) => Ok(game_map.await.unwrap()), // TODO handle properly
        Err(error) => {
            return Err(format!(
                "{}\nBacktrace:\n{}",
                error
                    .downcast::<&str>()
                    .unwrap_or(Box::new("Unknown Source of Error")),
                Backtrace::force_capture().to_string()
            ));
        }
    };
}

fn main() {
    #[cfg(debug_assertions)]
    ts::export(
        collect_types![get_game_map, get_owned_games],
        "../frontend/api/bindings.ts",
    )
    .unwrap();

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![get_game_map, get_owned_games])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
