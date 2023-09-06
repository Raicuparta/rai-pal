// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use game::GameMap;
use specta::collect_types;
use tauri_specta::ts;

mod appinfo;
mod game;
mod game_executable;
mod steam_game;

#[tauri::command]
#[specta::specta]
async fn get_game_map() -> GameMap {
    return steam_game::get_steam_apps();
}

fn main() {
    #[cfg(debug_assertions)]
    ts::export(collect_types![get_game_map], "../src/api/bindings.ts").unwrap();

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![get_game_map])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
