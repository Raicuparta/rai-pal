// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[path = "vdf.rs"] mod vdf;

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn greet(name: &str) -> String {
    let app_info = vdf::read_appinfo("/home/rai/.steam/steam/appcache/appinfo.vdf");

    return format!("Hello, {}! You've been greeted from Rust!", app_info.apps.len());
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
