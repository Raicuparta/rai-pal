use slint::ComponentHandle as _;
use std::{
	collections::HashMap,
	fs,
	path::{Path, PathBuf},
	sync::{
		Arc, Mutex,
		atomic::{AtomicBool, Ordering},
	},
	thread,
	time::Duration,
};

use base64::{Engine as _, engine::general_purpose::STANDARD};
use minisign_verify::{PublicKey, Signature};
#[cfg(target_os = "windows")]
use rai_pal_core::app_paths;
use rai_pal_core::result::LogErrExt;
use semver::Version;
use serde::Deserialize;
use slint::{Timer, TimerMode};

use crate::{MainWindow, fetch, notifications};

const LATEST_JSON_URL: &str =
	"https://github.com/Raicuparta/rai-pal/releases/latest/download/latest.json";
const PUBLIC_KEY: &str = "dW50cnVzdGVkIGNvbW1lbnQ6IG1pbmlzaWduIHB1YmxpYyBrZXk6IDNGMzI5QUM1RjMwMjM0OTMKUldTVE5BTHp4Wm95UDcvaWgyKzRENVkzZVc2K1JTOEhidUR4OFVENWVqemloUWRmZDllbzJGaSsK";
const CHECK_INTERVAL: Duration = Duration::from_secs(120);

#[derive(Clone)]
struct Release {
	version: String,
	notes: String,
	url: String,
	signature: String,
}

#[derive(Deserialize)]
struct LatestRelease {
	version: String,
	#[serde(default)]
	notes: String,
	platforms: HashMap<String, PlatformRelease>,
}

#[derive(Deserialize)]
struct PlatformRelease {
	signature: String,
	url: String,
}

pub struct Updater {
	pending: Arc<Mutex<Option<Release>>>,
	ignored: Arc<AtomicBool>,
	_timer: Timer,
}

impl Updater {
	pub fn start(window: slint::Weak<MainWindow>) -> Self {
		let pending = Arc::new(Mutex::new(None::<Release>));
		let ignored = Arc::new(AtomicBool::new(false));
		let checking = Arc::new(AtomicBool::new(false));

		let check = {
			let window = window.clone();
			let pending = pending.clone();
			let ignored = ignored.clone();
			let checking = checking.clone();

			move || check_for_update(&window, &pending, &ignored, &checking)
		};

		check();

		let timer = Timer::default();
		timer.start(TimerMode::Repeated, CHECK_INTERVAL, move || check());

		Self {
			pending,
			ignored,
			_timer: timer,
		}
	}

	pub fn install(&self, window: slint::Weak<MainWindow>) {
		let Some(release) = self
			.pending
			.lock()
			.ok_or_log("Failed to lock pending update")
			.and_then(|pending| pending.clone())
		else {
			return;
		};

		if let Some(window) = window.upgrade() {
			window.global::<crate::UpdateState>().set_installing(true);
		}

		thread::spawn(move || {
			let result =
				download_and_verify(&release).and_then(|bytes| install_and_restart(&bytes));

			if let Err(err) = result {
				log::error!("Failed to install app update: {err}");

				crate::runtime::invoke_on_ui(move || {
					let Some(window) = window.upgrade() else {
						return;
					};

					window.global::<crate::UpdateState>().set_installing(false);
					notifications::error(&window, format!("Failed to install app update: {err}"));
				});

				// Keep the update available so the user can retry.
			}
		});
	}

	pub fn ignore(&self, window: &MainWindow) {
		self.ignored.store(true, Ordering::SeqCst);
		match self.pending.lock() {
			Ok(mut pending) => {
				pending.take();
			}
			Err(err) => log::warn!("Failed to lock pending update: {err}"),
		}
		window.global::<crate::UpdateState>().set_open(false);
	}
}

fn check_for_update(
	window: &slint::Weak<MainWindow>,
	pending: &Arc<Mutex<Option<Release>>>,
	ignored: &Arc<AtomicBool>,
	checking: &Arc<AtomicBool>,
) {
	let has_pending = pending
		.lock()
		.ok_or_log("Failed to lock pending update")
		.is_some_and(|pending| pending.is_some());
	if ignored.load(Ordering::SeqCst) || has_pending || checking.swap(true, Ordering::SeqCst) {
		return;
	}

	let window = window.clone();
	let pending = pending.clone();
	let ignored = ignored.clone();
	let checking = checking.clone();

	thread::spawn(move || {
		let result = fetch_update();

		crate::runtime::invoke_on_ui(move || {
			checking.store(false, Ordering::SeqCst);

			let Some(window) = window.upgrade() else {
				return;
			};

			match result {
				Ok(Some(release)) => {
					let mut pending = match pending.lock() {
						Ok(pending) => pending,
						Err(err) => {
							log::warn!("Failed to lock pending update: {err}");
							return;
						}
					};

					if ignored.load(Ordering::SeqCst) || pending.is_some() {
						return;
					}

					window
						.global::<crate::UpdateState>()
						.set_version(release.version.clone().into());
					window
						.global::<crate::UpdateState>()
						.set_notes(release.notes.clone().into());
					window.global::<crate::UpdateState>().set_installing(false);
					window.global::<crate::UpdateState>().set_open(true);
					*pending = Some(release);
				}
				Ok(None) => {}
				Err(err) => log::warn!("Failed to check for app updates: {err}"),
			}
		});
	});
}

fn fetch_update() -> Result<Option<Release>, String> {
	let release = fetch::http_client()
		.get(LATEST_JSON_URL)
		.send()
		.map_err(|err| err.to_string())?
		.error_for_status()
		.map_err(|err| err.to_string())?
		.json::<LatestRelease>()
		.map_err(|err| err.to_string())?;

	let current = Version::parse(env!("CARGO_PKG_VERSION")).map_err(|err| err.to_string())?;
	let remote =
		Version::parse(release.version.trim_start_matches('v')).map_err(|err| err.to_string())?;

	if remote <= current {
		return Ok(None);
	}

	let Some(platform_key) = platform_key() else {
		return Ok(None);
	};

	let platform = release
		.platforms
		.get(platform_key)
		.ok_or_else(|| format!("No update available for {platform_key}"))?;

	Ok(Some(Release {
		version: release.version,
		notes: release.notes,
		url: platform.url.clone(),
		signature: platform.signature.clone(),
	}))
}

fn platform_key() -> Option<&'static str> {
	if cfg!(all(target_os = "linux", target_arch = "x86_64")) {
		Some("linux-x86_64")
	} else if cfg!(all(target_os = "windows", target_arch = "x86_64")) {
		Some("windows-x86_64")
	} else {
		None
	}
}

fn download_and_verify(release: &Release) -> Result<Vec<u8>, String> {
	let bytes = reqwest::blocking::Client::builder()
		.timeout(Duration::from_secs(600))
		.build()
		.map_err(|err| err.to_string())?
		.get(&release.url)
		.send()
		.map_err(|err| err.to_string())?
		.error_for_status()
		.map_err(|err| err.to_string())?
		.bytes()
		.map_err(|err| err.to_string())?
		.to_vec();

	verify_signature(&bytes, &release.signature)?;
	Ok(bytes)
}

fn verify_signature(data: &[u8], signature_b64: &str) -> Result<(), String> {
	let public_key_text =
		String::from_utf8(STANDARD.decode(PUBLIC_KEY).map_err(|err| err.to_string())?)
			.map_err(|err| err.to_string())?;
	let public_key = PublicKey::decode(&public_key_text).map_err(|err| err.to_string())?;

	let signature_text = String::from_utf8(
		STANDARD
			.decode(signature_b64)
			.map_err(|err| err.to_string())?,
	)
	.map_err(|err| err.to_string())?;
	let signature = Signature::decode(&signature_text).map_err(|err| err.to_string())?;

	public_key
		.verify(data, &signature, true)
		.map_err(|err| err.to_string())
}

#[cfg(target_os = "windows")]
fn update_file_path(file_name: &str) -> Result<PathBuf, String> {
	let directory = app_paths::temp_dir("update").map_err(|err| err.to_string())?;
	Ok(directory.join(file_name))
}

#[cfg(target_os = "linux")]
fn install_and_restart(bytes: &[u8]) -> Result<(), String> {
	use std::os::unix::fs::PermissionsExt;

	let target = match std::env::var_os("APPIMAGE") {
		Some(path) => PathBuf::from(path),
		None => std::env::current_exe()
			.map_err(|err| format!("Could not find the AppImage path: {err}"))?,
	};
	let target = fs::canonicalize(&target).unwrap_or_else(|err| {
		log::warn!(
			"Failed to canonicalize AppImage path {}: {err}",
			target.display()
		);
		target
	});
	let parent = target
		.parent()
		.ok_or_else(|| "Could not find the AppImage folder".to_string())?;
	let file_name = target
		.file_name()
		.and_then(|name| name.to_str())
		.unwrap_or("rai-pal.AppImage");
	let temp_path = parent.join(format!(".{file_name}.update"));

	fs::write(&temp_path, bytes).map_err(|err| err.to_string())?;
	fs::set_permissions(&temp_path, fs::Permissions::from_mode(0o755))
		.map_err(|err| err.to_string())?;
	fs::rename(&temp_path, &target).map_err(|err| err.to_string())?;

	spawn_relaunch(&target)?;
	std::process::exit(0);
}

#[cfg(target_os = "linux")]
fn spawn_relaunch(target: &Path) -> Result<(), String> {
	use std::process::Command;

	let pid = std::process::id();
	let target = shell_quote(target);
	let script = format!("while kill -0 {pid} 2>/dev/null; do sleep 0.1; done; exec {target}");
	Command::new("sh")
		.args(["-c", &script])
		.spawn()
		.map(|_| ())
		.map_err(|err| err.to_string())
}

#[cfg(target_os = "linux")]
fn shell_quote(path: &Path) -> String {
	format!("'{}'", path.to_string_lossy().replace('\'', "'\\''"))
}

#[cfg(target_os = "windows")]
fn install_and_restart(bytes: &[u8]) -> Result<(), String> {
	use std::{os::windows::process::CommandExt, process::Command};

	const CREATE_NO_WINDOW: u32 = 0x0800_0000;

	let installer = update_file_path("RaiPal-Setup.exe")?;
	fs::write(&installer, bytes).map_err(|err| err.to_string())?;

	let pid = std::process::id();
	let installer = installer.display();
	let script = format!(
		"@echo off\r\n:wait\r\ntasklist /FI \"PID eq {pid}\" /NH | find \"{pid}\" >nul\r\nif not errorlevel 1 (\r\n  timeout /t 1 /nobreak >nul\r\n  goto wait\r\n)\r\n\"{installer}\" /S /UPDATER\r\n"
	);

	Command::new("cmd")
		.args(["/C", &script])
		.creation_flags(CREATE_NO_WINDOW)
		.spawn()
		.map_err(|err| err.to_string())?;

	std::process::exit(0);
}

#[cfg(not(any(target_os = "linux", target_os = "windows")))]
fn install_and_restart(_bytes: &[u8]) -> Result<(), String> {
	Err("Automatic updates are not supported on this platform".to_string())
}
