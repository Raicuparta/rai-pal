use std::collections::HashMap;

use rai_pal_backend::{
	app_paths, game_providers::game_provider::GameProviderId,
	local_database::game_database::GameDatabase, result::LogErrExt,
};
use url::Url;

use super::CommandHandler;

#[derive(Clone, Copy, Debug)]
enum ModAction {
	Install,
	Uninstall,
	Run,
}

impl CommandHandler {
	pub fn handle_deep_link(&self, raw_url: &str) {
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
					self.prompt_add_mod_source(source_url);
				}
			}
			"install-mod" => self.dispatch_mod_action(ModAction::Install, &query),
			"uninstall-mod" => self.dispatch_mod_action(ModAction::Uninstall, &query),
			"run-mod" => self.dispatch_mod_action(ModAction::Run, &query),
			_ => log::warn!("Ignoring unknown rai-pal:// path `{action}`"),
		}
	}

	fn dispatch_mod_action(&self, action: ModAction, query: &HashMap<String, String>) {
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

		let Some(game) = self
			.database
			.get_game(&provider_id, game_id)
			.ok_or_log("Failed to load game from deep link")
		else {
			log::warn!("Ignoring rai-pal:// {action:?} deep link for an unknown game");
			return;
		};

		let mod_id = mod_id.clone();

		match action {
			ModAction::Install => self.install_mod(game, mod_id),
			ModAction::Uninstall => self.uninstall_mod(game, mod_id),
			ModAction::Run => self.run_mod(game, mod_id),
		}
	}
}

pub fn register() {
	#[cfg(target_os = "linux")]
	register_linux();
	#[cfg(target_os = "windows")]
	register_windows();
	#[cfg(target_os = "macos")]
	register_macos();
}

#[cfg(target_os = "macos")]
fn register_macos() {
	use std::ffi::c_void;

	const K_INTERNET_EVENT_CLASS: u32 = 0x4755_524C;
	const K_AE_GET_URL: u32 = 0x4755_524C;

	type EventHandler = unsafe extern "C" fn(*const c_void, *mut c_void, isize) -> i16;

	#[link(name = "Carbon", kind = "framework")]
	unsafe extern "C" {
		fn AEInstallEventHandler(
			event_class: u32,
			event_id: u32,
			handler: Option<EventHandler>,
			handler_refcon: isize,
			is_system_handler: u8,
		) -> i32;
	}

	let status = unsafe {
		AEInstallEventHandler(
			K_INTERNET_EVENT_CLASS,
			K_AE_GET_URL,
			Some(handle_macos_url_event),
			0,
			0,
		)
	};

	if status != 0 {
		log::warn!("Failed to register macOS URL event handler: {status}");
	}
}

#[cfg(target_os = "macos")]
unsafe extern "C" fn handle_macos_url_event(
	event: *const std::ffi::c_void,
	_reply: *mut std::ffi::c_void,
	_refcon: isize,
) -> i16 {
	use std::ffi::c_void;

	const KEY_DIRECT_OBJECT: u32 = 0x2D2D_2D2D;
	const TYPE_UTF8_TEXT: u32 = 0x7574_6638;

	#[link(name = "Carbon", kind = "framework")]
	unsafe extern "C" {
		fn AEGetParamPtr(
			event: *const c_void,
			keyword: u32,
			desired_type: u32,
			actual_type: *mut u32,
			data: *mut c_void,
			max_size: usize,
			actual_size: *mut usize,
		) -> i16;
	}

	let mut buffer = [0_u8; 16 * 1024];
	let mut actual_size = 0_usize;
	let status = unsafe {
		AEGetParamPtr(
			event,
			KEY_DIRECT_OBJECT,
			TYPE_UTF8_TEXT,
			std::ptr::null_mut(),
			buffer.as_mut_ptr().cast(),
			buffer.len(),
			&mut actual_size,
		)
	};

	if status != 0 {
		return status;
	}

	if let Ok(url) = std::str::from_utf8(&buffer[..actual_size as usize]) {
		let url = url.to_string();
		crate::runtime::invoke_on_ui(move || {
			let Some(handler) = CommandHandler::current() else {
				return;
			};
			handler.handle_deep_link(&url);
		});
	}

	0
}

#[cfg(target_os = "linux")]
fn register_linux() {
	use std::process::Command;

	use fs_err as fs;

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

	let Some(executable) = app_paths::exe_path().ok_or_log("Failed to get current executable path")
	else {
		return;
	};
	// Deep links only need a hidden handler, the visible app entry with its icon ships
	// inside the AppImage's own `.desktop` file.
	let desktop_file = applications_dir.join("Rai-Pal-handler.desktop");
	let contents = format!(
		"[Desktop Entry]\nType=Application\nName=Rai Pal\nNoDisplay=true\nExec=\"{}\" %u\nTerminal=false\nMimeType=x-scheme-handler/rai-pal;\n",
		executable.display()
	);

	if let Err(err) = fs::write(&desktop_file, contents) {
		log::warn!("Failed to write deep link desktop file: {err}");
		return;
	}

	match Command::new("xdg-mime")
		.args([
			"default",
			"Rai-Pal-handler.desktop",
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

	let Some(executable) = app_paths::exe_path().ok_or_log("Failed to get current executable path")
	else {
		return;
	};

	let result = (|| -> std::io::Result<()> {
		let hkcu = RegKey::predef(HKEY_CURRENT_USER);
		let (key, _) = hkcu.create_subkey(r"Software\Classes\rai-pal")?;
		key.set_value("", &"URL:Rai Pal Protocol".to_string())?;
		key.set_value("URL Protocol", &"")?;

		let (icon, _) = key.create_subkey("DefaultIcon")?;
		icon.set_value("", &format!("\"{}\"", executable.display()))?;

		let (command, _) = key.create_subkey(r"shell\open\command")?;
		command.set_value("", &format!("\"{}\" \"%1\"", executable.display()))?;

		Ok(())
	})();

	if let Err(err) = result {
		log::warn!("Failed to register deep link protocol: {err}");
	}
}
