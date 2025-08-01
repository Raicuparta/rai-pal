#![allow(async_fn_in_trait)]

pub mod analytics;
pub mod architecture;
pub mod data_types;
pub mod debug;
pub mod files;
pub mod game;
pub mod game_engines;
pub mod game_mod;
pub mod game_tag;
pub mod game_title;
pub mod games_query;
pub mod local_database;
pub mod local_mod;
pub mod maps;
pub mod mod_loaders;
pub mod mod_manifest;
pub mod paths;
pub mod providers;
pub mod remote_config;
pub mod remote_game;
pub mod remote_mod;
pub mod result;
pub mod windows;

#[cfg(test)]
mod tests;
