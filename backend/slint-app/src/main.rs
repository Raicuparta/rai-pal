#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

slint::include_modules!();

mod actions;
mod app;
mod fetch;
mod filters;
mod i18n;
mod mod_sources;
mod model;
mod mods;
mod mods_tab;
mod query;
mod refresh;
mod scheduler;
mod selected;
mod tools;
mod user;

fn main() -> Result<(), slint::PlatformError> {
	app::run()
}
