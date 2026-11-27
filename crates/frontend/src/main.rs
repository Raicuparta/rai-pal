#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// Slint generates a large amount of code that we don't control; don't lint it.
#[allow(clippy::all, clippy::pedantic, clippy::nursery, clippy::restriction)]
mod generated {
	slint::include_modules!();
}

pub use generated::*;

mod action_state;
mod analytics;
mod app;
mod commands;
mod convert;
mod downloads;
mod fetch;
mod filters;
mod i18n;
mod image_cache;
mod languages;
mod logging;
mod mod_actions;
mod model;
mod notifications;
mod runtime;
mod scheduler;
mod selected;
mod single_instance;
mod text;
mod ui_models;
mod window_state;

fn main() -> Result<(), slint::PlatformError> {
	#[cfg(feature = "dev")]
	unsafe {
		std::env::set_var(
			"SLINT_ENABLE_EXPERIMENTAL_FEATURES",
			env!("SLINT_PROJECT_EXPERIMENTAL_FEATURES"),
		);
	}

	app::run()
}
