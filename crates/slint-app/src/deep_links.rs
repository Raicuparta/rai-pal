use std::{collections::HashMap, sync::Arc};

use rai_pal_core::{
	app_paths,
	game_providers::game_provider::GameProviderId,
	local_database::{app_database::DbMutex, game_database::GameDatabase},
	result::LogErrExt,
};
use slint::ComponentHandle;
use url::Url;

use crate::{MainWindow, downloads::Tracker, mod_sources, mods, selected::SelectedGameState};

#[derive(Clone, Copy, Debug)]
enum ModAction {
	Install,
	Uninstall,
	Run,
}

pub fn handle(
	raw_url: &str,
	window: &MainWindow,
	database: &Arc<DbMutex>,
	selected_game: &SelectedGameState,
	downloads: &Tracker,
) {
	let Some(url) = Url::parse(raw_url).ok_or_log("Failed to parse deep link URL") else {
		return;
	};

	if url.scheme() != "rai-pal" {
		return;
	}

	let Some(action) = url.host_str() else {
		return;
	};

	let query = url.query_pairs().into_owned().collect::<HashMap<_, _>>();

	match action {
		"add-mod-source" => {
			if let Some(source_url) = query.get("url") {
				mod_sources::request_add(window, source_url.clone());
			}
		}
		"install-mod" => dispatch_mod_action(
			ModAction::Install,
			&query,
			window,
			database,
			selected_game,
			downloads,
		),
		"uninstall-mod" => dispatch_mod_action(
			ModAction::Uninstall,
			&query,
			window,
			database,
			selected_game,
			downloads,
		),
		"run-mod" => dispatch_mod_action(
			ModAction::Run,
			&query,
			window,
			database,
			selected_game,
			downloads,
		),
		_ => log::warn!("Ignoring unknown rai-pal:// path `{action}`"),
	}
}

fn dispatch_mod_action(
	action: ModAction,
	query: &HashMap<String, String>,
	window: &MainWindow,
	database: &Arc<DbMutex>,
	selected_game: &SelectedGameState,
	downloads: &Tracker,
) {
	let Some(provider_id) = query.get("providerId") else {
		log::warn!("Ignoring rai-pal:// {action:?} deep link without a providerId");
		return;
	};
	let Some(provider_id) = provider_id
		.parse::<GameProviderId>()
		.ok_or_log("Failed to parse deep link provider ID")
	else {
		log::warn!("Ignoring rai-pal:// {action:?} deep link with an invalid providerId");
		return;
	};

	let (Some(game_id), Some(mod_id)) = (query.get("gameId"), query.get("modId")) else {
		log::warn!("Ignoring rai-pal:// {action:?} deep link without gameId or modId");
		return;
	};

	let Some(game) = database
		.get_game(&provider_id, game_id)
		.ok_or_log("Failed to load game from deep link")
	else {
		log::warn!("Ignoring rai-pal:// {action:?} deep link for an unknown game");
		return;
	};

	let window = window.as_weak();
	let mod_id = mod_id.clone();

	match action {
		ModAction::Install => {
			mods::install_mod(database.clone(), window, game, mod_id, downloads.clone());
		}
		ModAction::Uninstall => {
			mods::uninstall_mod(database.clone(), window, game, mod_id);
		}
		ModAction::Run => {
			mods::run_mod(database.clone(), window, game, mod_id);
		}
	}

	// Keep the reference used so callers can select the game later if needed.
	let _ = selected_game;
}

pub fn register() {
	#[cfg(target_os = "linux")]
	register_linux();
	#[cfg(target_os = "windows")]
	register_windows();
}

#[cfg(target_os = "linux")]
fn register_linux() {
	use std::{fs, process::Command};

	let Some(dirs) =
		app_paths::base_dirs().ok_or_log("Failed to get base dirs for deep link registration")
	else {
		return;
	};
	let applications_dir = dirs.data_dir().join("applications");
	if let Err(err) = fs::create_dir_all(&applications_dir) {
		log::warn!("Failed to create applications directory for deep links: {err}");
		return;
	}

	let executable = current_executable();
	let desktop_file = applications_dir.join("rai-pal-slint.desktop");
	let contents = format!(
		"[Desktop Entry]\nType=Application\nName=Rai Pal\nComment=Universal game mod manager\nExec=\"{}\" %u\nIcon=rai-pal-slint\nTerminal=false\nCategories=Game;\nStartupNotify=true\nMimeType=x-scheme-handler/rai-pal;\n",
		executable.display()
	);

	if let Err(err) = fs::write(&desktop_file, contents) {
		log::warn!("Failed to write deep link desktop file: {err}");
		return;
	}

	match Command::new("xdg-mime")
		.args([
			"default",
			"rai-pal-slint.desktop",
			"x-scheme-handler/rai-pal",
		])
		.status()
	{
		Ok(status) if status.success() => {}
		Ok(status) => log::warn!("xdg-mime exited with {status} while registering deep links"),
		Err(err) => log::warn!("Failed to run xdg-mime for deep link registration: {err}"),
	}
}

#[cfg(target_os = "windows")]
fn register_windows() {
	use winreg::{RegKey, enums::HKEY_CURRENT_USER};

	let result = (|| -> std::io::Result<()> {
		let hkcu = RegKey::predef(HKEY_CURRENT_USER);
		let (key, _) = hkcu.create_subkey(r"Software\Classes\rai-pal")?;
		key.set_value("", &"URL:Rai Pal Protocol".to_string())?;
		key.set_value("URL Protocol", &"")?;

		let (icon, _) = key.create_subkey("DefaultIcon")?;
		icon.set_value("", &format!("\"{}\"", current_executable().display()))?;

		let (command, _) = key.create_subkey(r"shell\open\command")?;
		command.set_value(
			"",
			&format!("\"{}\" \"%1\"", current_executable().display()),
		)?;

		Ok(())
	})();

	if let Err(err) = result {
		log::warn!("Failed to register deep link protocol: {err}");
	}
}

fn current_executable() -> std::path::PathBuf {
	std::env::var_os("APPIMAGE")
		.map(std::path::PathBuf::from)
		.or_else(|| std::env::current_exe().ok_or_log("Failed to get current executable path"))
		.unwrap_or_default()
}
