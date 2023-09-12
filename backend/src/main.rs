// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![deny(clippy::all)]

use anyhow::anyhow;
use std::result::Result as StdResult;
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

#[tauri::command]
#[specta::specta]
async fn get_game_map() -> CommandResult<GameMap> {
    return match panic::catch_unwind(get_steam_games) {
        Ok(game_map) => Ok(game_map),
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
async fn get_owned_games() -> CommandResult<Vec<OwnedUnityGame>> {
    return match panic::catch_unwind(get_steam_owned_unity_games) {
        Ok(game_map) => Ok(game_map.await.unwrap()), // TODO handle properly
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
