// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{backtrace::Backtrace, panic};

use game::GameMap;
use specta::collect_types;
use tauri_specta::ts;

mod appinfo;
mod game;
mod game_executable;
mod steam_game;

#[tauri::command]
#[specta::specta]
fn get_game_map() -> Result<GameMap, String> {
    return match panic::catch_unwind(|| steam_game::get_steam_apps()) {
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

fn main() {
    #[cfg(debug_assertions)]
    ts::export(collect_types![get_game_map], "../frontend/api/bindings.ts").unwrap();

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![get_game_map])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
