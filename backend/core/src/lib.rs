#![allow(async_fn_in_trait)]

pub mod analytics;
pub mod app_paths;
pub mod architecture;
pub mod data_types;
pub mod debug;
pub mod files;
pub mod game;
pub mod game_engines;
pub mod game_mods;
pub mod game_tag;
pub mod game_title;
pub mod games_query;
pub mod http;
pub mod local_database;
pub mod maps;
pub mod open_better;
pub mod operating_system;
pub mod path_extensions;
pub mod providers;
pub mod remote_config;
pub mod remote_game;
pub mod result;
pub mod user;
pub mod windows;
pub mod wine;

#[cfg(test)]
mod tests;
