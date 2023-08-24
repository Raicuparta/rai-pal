// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::collections::HashMap;

use serde::Serialize;
use steamlocate::SteamDir;
use vdf::LaunchMap;

#[path = "vdf.rs"]
mod vdf;

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn greet(name: &str) -> String {
    let app_details_map = get_app_details();

    return serde_json::to_string_pretty(&app_details_map).unwrap();
}

#[derive(Serialize)]
struct SteamApp {
    name: String,
    launch_map: LaunchMap,
    install_path: String,
}

fn get_app_details() -> HashMap<u32, SteamApp> {
    let mut steam_dir = SteamDir::locate().unwrap();
    let app_info = vdf::read_appinfo(
        &steam_dir
            .path
            .join("appcache/appinfo.vdf")
            .to_string_lossy(),
    );

    let mut app_details_map: HashMap<u32, SteamApp> = HashMap::new();
    for (app_id, app_option) in steam_dir.apps() {
        if let Some(app) = app_option {
            if let Some(launch_map) = app_info.apps.get(&app_id) {
                app_details_map.insert(
                    app_id.to_owned(),
                    SteamApp {
                        name: app.name.clone().unwrap_or_default(),
                        launch_map: launch_map.clone(),
                        install_path: app.path.to_string_lossy().to_string(),
                    },
                );
            }
        }
    }

    app_details_map
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
