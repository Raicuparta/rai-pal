// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use appinfo::SteamLaunchOption;
use serde::Serialize;
use specta::collect_types;
use specta::Type;
use std::{collections::HashMap, fs, path::PathBuf};
use steamlocate::SteamDir;
use tauri_specta::ts;

#[path = "appinfo.rs"]
mod appinfo;

type GameMap = HashMap<u32, Game>;

#[tauri::command]
#[specta::specta]
async fn get_steam_apps_json() -> GameMap {
    return get_steam_apps();
}

#[derive(Serialize, Type)]
#[serde(rename_all = "camelCase")]
struct GameExecutable {
    id: String,
    name: String,
    is_legacy: bool,
    mod_files_path: String,
    full_path: PathBuf,
    architecture: String,
    scripting_backend: String,
    unity_version: String,
    is_linux: bool,
    steam_launch: Option<SteamLaunchOption>,
}

#[derive(Serialize, Type)]
#[serde(rename_all = "camelCase")]
struct Game {
    id: u32,
    name: String,
    executables: Vec<GameExecutable>,
    distinct_executables: Vec<GameExecutable>,
}

fn get_steam_apps() -> GameMap {
    let mut steam_dir = SteamDir::locate().unwrap();
    let app_info = appinfo::read_appinfo(
        &steam_dir
            .path
            .join("appcache/appinfo.vdf")
            .to_string_lossy(),
    );

    let mut app_details_map: GameMap = HashMap::new();
    for (app_id, app_option) in steam_dir.apps() {
        if let Some(app) = app_option {
            if let Some(steam_launch_options) = app_info.apps.get(&app_id) {
                let id = app_id.to_owned();

                app_details_map.insert(
                    id,
                    Game {
                        id,
                        name: app.name.clone().unwrap_or_default(),
                        distinct_executables: steam_launch_options
                            .iter()
                            .filter_map(|launch_option| {
                                if let Some(executable) = &launch_option.executable {
                                    let full_path = app.path.join(executable);

                                    if !is_unity_game(&full_path) {
                                        return None;
                                    }

                                    return Some(GameExecutable {
                                        architecture: String::from("x64"),
                                        full_path,
                                        id: launch_option.launch_id.clone(),
                                        is_legacy: false,
                                        is_linux: false,
                                        mod_files_path: String::from(""),
                                        name: executable.clone(),
                                        scripting_backend: String::from("il2cpp"),
                                        steam_launch: Some(launch_option.clone()),
                                        unity_version: String::from("2020"),
                                    });
                                }
                                return None;
                            })
                            .collect(),
                        executables: Vec::new(), // TODO distinguish them!
                    },
                );
            }
        }
    }

    app_details_map
}

fn is_unity_game(game_exe_path: &PathBuf) -> bool {
    if let Ok(data_path) = get_data_path(game_exe_path) {
        fs::metadata(game_exe_path).is_ok() && fs::metadata(data_path).is_ok()
    } else {
        false
    }
}

fn file_name_without_extension(file_path: &PathBuf) -> Option<String> {
    file_path
        .file_stem()
        .and_then(|stem| stem.to_str().map(|s| s.to_string()))
}

fn get_data_path(game_exe_path: &PathBuf) -> Result<PathBuf, &'static str> {
    if let Some(parent) = game_exe_path.parent() {
        if let Some(exe_name) = file_name_without_extension(game_exe_path) {
            let data_folder_name = format!("{}_Data", exe_name);
            Ok(parent.join(data_folder_name))
        } else {
            Err("Failed to get file name without extension")
        }
    } else {
        Err("Failed to get parent directory")
    }
}

fn main() {
    #[cfg(debug_assertions)]
    ts::export(
        collect_types![get_steam_apps_json],
        "../src/api/bindings.ts",
    )
    .unwrap();

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![get_steam_apps_json])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
