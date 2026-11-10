use rai_pal_backend::{app_paths, result::LogErrExt};

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
