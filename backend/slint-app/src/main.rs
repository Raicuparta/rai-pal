#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

slint::include_modules!();

mod actions;
mod app;
mod fetch;
mod filters;
mod i18n;
mod model;
mod mods;
mod mods_tab;
mod query;
mod scheduler;
mod selected;

fn main() -> Result<(), slint::PlatformError> {
	app::run()
}
