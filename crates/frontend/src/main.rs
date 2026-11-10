#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

slint::include_modules!();

mod action_state;
mod actions;
mod analytics;
mod app;
mod confirmation;
mod deep_link_registration;
mod deep_links;
mod downloads;
mod fetch;
mod filters;
mod i18n;
mod image_cache;
mod logging;
mod manual;
mod mod_sources;
mod model;
mod mods;
mod mods_tab;
mod notifications;
mod query;
mod refresh;
mod runtime;
mod scheduler;
mod selected;
mod single_instance;
mod text;
mod thanks;
mod tools;
mod updater;
mod user;

fn main() -> Result<(), slint::PlatformError> {
	app::run()
}
