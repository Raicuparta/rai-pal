#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

slint::include_modules!();

mod actions;
mod app;
mod confirmation;
mod fetch;
mod filters;
mod i18n;
mod manual;
mod mod_sources;
mod model;
mod mods;
mod mods_tab;
mod notifications;
mod query;
mod refresh;
mod scheduler;
mod selected;
mod thanks;
mod tools;
mod user;

fn main() -> Result<(), slint::PlatformError> {
	app::run()
}
