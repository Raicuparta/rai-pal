// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::collections::HashMap;

use appinfo::SteamLaunchOption;
use serde::Serialize;
use steamlocate::SteamDir;

#[path = "appinfo.rs"]
mod appinfo;

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
async fn get_steam_apps_json() -> String {
    let steam_apps = get_steam_apps();

    return serde_json::to_string_pretty(&steam_apps).unwrap();
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct GameExecutable {
    id: String,
    name: String,
    is_legacy: bool,
    mod_files_path: String,
    full_path: String,
    architecture: String,
    scripting_backend: String,
    unity_version: String,
    is_linux: bool,
    steam_launch: Option<SteamLaunchOption>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Game {
    id: u32,
    name: String,
    executables: Vec<GameExecutable>,
    distinct_executables: Vec<GameExecutable>,
}

fn get_steam_apps() -> HashMap<u32, Game> {
    let mut steam_dir = SteamDir::locate().unwrap();
    let app_info = appinfo::read_appinfo(
        &steam_dir
            .path
            .join("appcache/appinfo.vdf")
            .to_string_lossy(),
    );

    let mut app_details_map: HashMap<u32, Game> = HashMap::new();
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
                            .into_iter()
                            .map(|launch_option| {
                                GameExecutable {
                                    architecture: String::from("x64"),
                                    full_path: launch_option.executable.clone().unwrap_or_default(), // TODO full path!
                                    id: launch_option.launch_id.clone(),
                                    is_legacy: false,
                                    is_linux: false,
                                    mod_files_path: String::from(""),
                                    name: launch_option.executable.clone().unwrap_or_default(),
                                    scripting_backend: String::from("il2cpp"),
                                    steam_launch: Some(launch_option.clone()),
                                    unity_version: String::from("2020"),
                                }
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

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![get_steam_apps_json])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
