// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{collections::HashMap, fs};

use library_folders::{LibraryFolder, LibraryFolders};
use serde::Serialize;
use vdf::AppDetails;

#[path = "library_folders.rs"]
mod library_folders;
#[path = "vdf.rs"]
mod vdf;

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn greet(name: &str) -> String {
    let library_folders_result =
        fs::read_to_string("C:/Program Files (x86)/Steam/steamapps/libraryfolders.vdf");

    let libraries_result = match library_folders_result {
        Ok(library_folders_text) => {
            keyvalues_serde::from_str::<LibraryFolders>(library_folders_text.as_str())
        }
        Err(error) => panic!("Problem opening the file: {:?}", error),
    };

    let app_details_map = match libraries_result {
        Ok(libraries) => get_app_details(libraries),
        Err(error) => HashMap::new(), // TODO error
    };

    return serde_json::to_string_pretty(&app_details_map).unwrap();
}

#[derive(Serialize)]
struct SteamApp {
    details: AppDetails,
    library_path: String,
}

fn get_app_details(libraries: LibraryFolders) -> HashMap<u32, SteamApp> {
    let app_info = vdf::read_appinfo("C:/Program Files (x86)/Steam/appcache/appinfo.vdf");

    let mut app_details_map: HashMap<u32, SteamApp> = HashMap::new();

    for (_, library) in libraries {
        for app_id_text in library.apps.keys() {
            let app_id = app_id_text.parse::<u32>().unwrap();
            let config = app_info.apps.get(&app_id).unwrap().clone();

            app_details_map.insert(
                app_id,
                SteamApp {
                    details: config,
                    library_path: library.path.clone(),
                },
            );
        }
    }

    return app_details_map;
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
