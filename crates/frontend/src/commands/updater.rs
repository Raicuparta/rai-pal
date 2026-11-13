use fs_err as fs;
#[cfg(target_os = "linux")]
use rai_pal_backend::path_extensions::AsValidStr;
use slint::ComponentHandle as _;
#[cfg(any(target_os = "linux", target_os = "macos"))]
use std::path::Path;
#[cfg(target_os = "windows")]
use std::path::PathBuf;
use std::{
	collections::HashMap,
	sync::{
		Arc, Mutex,
		atomic::{AtomicBool, Ordering},
	},
	thread,
	time::Duration,
};

use base64::{Engine as _, engine::general_purpose::STANDARD};
use minisign_verify::{PublicKey, Signature};
#[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
use rai_pal_backend::app_paths;
use rai_pal_backend::{app_variant::AppVariant, result::LogErrExt};
use semver::Version;
use serde::Deserialize;
use slint::{Timer, TimerMode};

use crate::{MainWindow, MessageContext, fetch, notifications};

use super::on_command;

const LATEST_JSON_URL: &str =
	"https://github.com/Raicuparta/rai-pal/releases/latest/download/latest.json";
const LATEST_JSON_URL_ENV_VAR: &str = "RAI_PAL_UPDATE_URL";
const PUBLIC_KEY: &str = "dW50cnVzdGVkIGNvbW1lbnQ6IG1pbmlzaWduIHB1YmxpYyBrZXk6IDNGMzI5QUM1RjMwMjM0OTMKUldTVE5BTHp4Wm95UDcvaWgyKzRENVkzZVc2K1JTOEhidUR4OFVENWVqemloUWRmZDllbzJGaSsK";
const CHECK_INTERVAL: Duration = Duration::from_mins(2);

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
		#[cfg(target_os = "windows")]
		cleanup_portable_update_files();

		let pending = Arc::new(Mutex::new(None::<Release>));
		let ignored = Arc::new(AtomicBool::new(false));
		let checking = Arc::new(AtomicBool::new(false));

		let check = {
			let pending = Arc::clone(&pending);
			let ignored = Arc::clone(&ignored);

			move || check_for_update(&window, &pending, &ignored, &checking)
		};

		check();

		let timer = Timer::default();
		timer.start(TimerMode::Repeated, CHECK_INTERVAL, check);

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
			window
				.global::<crate::UpdateState>()
				.set_state(crate::ActionState::Loading);
		}

		thread::spawn(move || {
			let result =
				download_and_verify(&release).and_then(|bytes| install_and_restart(&bytes));

			if let Err(err) = result {
				log::error!("Failed to install app update: {err}");

				crate::runtime::with_ui_window(&window, move |window| {
					window
						.global::<crate::UpdateState>()
						.set_state(crate::ActionState::Idle);
					notifications::error(
						&window,
						notifications::data(MessageContext::InstallAppUpdate, &err),
					);
				});

				// Keep the update available so the user can retry.
			}
		});
	}

	pub fn ignore(&self, window: &MainWindow) {
		self.ignored.store(true, Ordering::SeqCst);
		window.global::<crate::UpdateState>().set_open(false);
	}
}

impl super::CommandHandler {
	pub fn install_update(&self) {
		self.updater.install(self.window());
	}

	pub fn ignore_update(&self) {
		if let Some(window) = self.ui() {
			self.updater.ignore(&window);
		}
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
	let pending = Arc::clone(pending);
	let ignored = Arc::clone(ignored);
	let checking = Arc::clone(checking);

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

					let update_state = window.global::<crate::UpdateState>();
					update_state.set_version(release.version.clone().into());
					update_state.set_notes(release.notes.clone().into());
					update_state.set_state(crate::ActionState::Idle);
					update_state.set_available(true);
					update_state.set_open(true);
					*pending = Some(release);
				}
				Ok(None) => {}
				Err(err) => log::warn!("Failed to check for app updates: {err}"),
			}
		});
	});
}

fn latest_json_url() -> String {
	std::env::var(LATEST_JSON_URL_ENV_VAR).unwrap_or_else(|_| LATEST_JSON_URL.to_string())
}

fn fetch_update() -> Result<Option<Release>, String> {
	let release = fetch::http_client()
		.get(latest_json_url())
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
	match AppVariant::get_current() {
		AppVariant::WindowsNsis => Some("windows-x86_64"),
		AppVariant::WindowsPortable => Some("windows-x86_64-portable"),
		AppVariant::LinuxAppImage => Some("linux-x86_64"),
		AppVariant::MacosBundle if cfg!(target_arch = "aarch64") => Some("darwin-aarch64"),
		AppVariant::MacosBundle => Some("darwin-x86_64"),
		AppVariant::Development | AppVariant::Unknown => None,
	}
}

fn download_and_verify(release: &Release) -> Result<Vec<u8>, String> {
	let bytes = reqwest::blocking::Client::builder()
		.timeout(Duration::from_mins(10))
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

#[cfg(target_os = "linux")]
fn install_and_restart(bytes: &[u8]) -> Result<(), String> {
	use std::os::unix::fs::PermissionsExt;

	use rai_pal_backend::files;

	let target =
		app_paths::exe_path().map_err(|err| format!("Could not find the AppImage path: {err}"))?;
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

	let update_dir = app_paths::temp_dir("update").map_err(|err| err.to_string())?;
	let archive_path = update_dir.join("updater-linux.tar.gz");
	fs::write(&archive_path, bytes).map_err(|err| err.to_string())?;

	let extract_dir = update_dir.join("extracted-linux");
	if extract_dir.exists() {
		fs::remove_dir_all(&extract_dir).map_err(|err| err.to_string())?;
	}
	files::extract_tar_gz(&archive_path, &extract_dir).map_err(|err| err.to_string())?;

	let appimage_path = fs::read_dir(&extract_dir)
		.map_err(|err| err.to_string())?
		.filter_map(|entry| entry.ok_or_log("Failed to read update directory entry"))
		.map(|entry| entry.path())
		.find(|path| {
			path.extension()
				.is_some_and(|extension| extension == "AppImage")
		})
		.ok_or_else(|| "Could not find the updated AppImage".to_string())?;

	fs::copy(&appimage_path, &temp_path).map_err(|err| err.to_string())?;
	fs::set_permissions(&temp_path, std::fs::Permissions::from_mode(0o755))
		.map_err(|err| err.to_string())?;
	fs::rename(&temp_path, &target).map_err(|err| err.to_string())?;

	spawn_relaunch(&target)?;
	std::process::exit(0);
}

#[cfg(target_os = "linux")]
fn spawn_relaunch(target: &Path) -> Result<(), String> {
	use std::process::Command;

	let pid = std::process::id();
	let target = shell_quote(target)?;
	let script = format!("while kill -0 {pid} 2>/dev/null; do sleep 0.1; done; exec {target}");
	crate::single_instance::release();
	Command::new("sh")
		.args(["-c", &script])
		.spawn()
		.map(|_| ())
		.map_err(|err| err.to_string())
}

#[cfg(target_os = "linux")]
fn shell_quote(path: &Path) -> Result<String, String> {
	let path = path.try_to_str().map_err(|err| err.to_string())?;
	Ok(format!("'{}'", path.replace('\'', "'\\''")))
}

#[cfg(target_os = "windows")]
fn extract_update_archive(
	bytes: &[u8],
	archive_name: &str,
	extract_dir_name: &str,
) -> Result<PathBuf, String> {
	use rai_pal_backend::files;

	let update_dir = app_paths::temp_dir("update").map_err(|err| err.to_string())?;
	let archive_path = update_dir.join(archive_name);
	fs::write(&archive_path, bytes).map_err(|err| err.to_string())?;

	let extract_dir = update_dir.join(extract_dir_name);
	if extract_dir.exists() {
		fs::remove_dir_all(&extract_dir).map_err(|err| err.to_string())?;
	}
	files::extract(&archive_path, &extract_dir, &|_, _| {}).map_err(|err| err.to_string())?;

	Ok(extract_dir)
}

#[cfg(target_os = "windows")]
fn install_and_restart(bytes: &[u8]) -> Result<(), String> {
	if AppVariant::get_current() == AppVariant::WindowsPortable {
		install_portable_and_restart(bytes)
	} else {
		install_installer_and_restart(bytes)
	}
}

#[cfg(target_os = "windows")]
fn install_installer_and_restart(bytes: &[u8]) -> Result<(), String> {
	use std::{ffi::OsStr, os::windows::ffi::OsStrExt};

	use windows_sys::Win32::UI::{Shell::ShellExecuteW, WindowsAndMessaging::SW_SHOWNORMAL};

	let extract_dir = extract_update_archive(bytes, "updater-windows.zip", "extracted-windows")?;
	let installer = extract_dir.join("RaiPal-Setup.exe");
	if !installer.is_file() {
		return Err("Could not find the updated installer".to_string());
	}

	let wide =
		|value: &OsStr| -> Vec<u16> { value.encode_wide().chain(std::iter::once(0)).collect() };

	let operation = wide(OsStr::new("open"));
	let installer = wide(installer.as_os_str());
	let parameters = wide(OsStr::new("/S /UPDATER"));

	let result = unsafe {
		ShellExecuteW(
			std::ptr::null_mut(),
			operation.as_ptr(),
			installer.as_ptr(),
			parameters.as_ptr(),
			std::ptr::null(),
			SW_SHOWNORMAL,
		)
	};

	if result as isize <= 32 {
		return Err(format!(
			"Failed to launch the update installer (ShellExecuteW returned {})",
			result as isize
		));
	}

	crate::single_instance::release();
	std::process::exit(0);
}

#[cfg(target_os = "windows")]
fn install_portable_and_restart(bytes: &[u8]) -> Result<(), String> {
	use std::process::Command;

	let extract_dir = extract_update_archive(
		bytes,
		"updater-windows-portable.zip",
		"extracted-windows-portable",
	)?;
	let new_exe = extract_dir.join("RaiPal-Portable.exe");
	if !new_exe.is_file() {
		return Err("Could not find the updated portable executable".to_string());
	}

	let (target, staged, backup) = portable_update_paths()?;

	cleanup_portable_update_files();
	fs::copy(&new_exe, &staged).map_err(|err| err.to_string())?;

	fs::rename(&target, &backup).map_err(|err| {
		format!(
			"Failed to move the current executable out of the way ({}): {err}",
			target.display()
		)
	})?;

	if let Err(err) = fs::rename(&staged, &target) {
		if let Err(rollback_err) = fs::rename(&backup, &target) {
			log::error!(
				"Failed to restore the previous executable after a failed update: {rollback_err}"
			);
		}
		return Err(format!(
			"Failed to move the updated executable into place ({}): {err}",
			target.display()
		));
	}

	crate::single_instance::release();
	Command::new(&target)
		.spawn()
		.map_err(|err| err.to_string())?;

	std::process::exit(0);
}

#[cfg(target_os = "windows")]
fn portable_update_paths() -> Result<(PathBuf, PathBuf, PathBuf), String> {
	let target =
		app_paths::exe_path().map_err(|err| format!("Could not find the app path: {err}"))?;
	let parent =
		app_paths::exe_folder().map_err(|err| format!("Could not find the app folder: {err}"))?;
	let file_name = target
		.file_name()
		.and_then(|name| name.to_str())
		.unwrap_or("Rai-Pal.exe");

	Ok((
		target.clone(),
		parent.join(format!(".{file_name}.update")),
		parent.join(format!(".{file_name}.old")),
	))
}

#[cfg(target_os = "windows")]
fn cleanup_portable_update_files() {
	let Ok((_, staged, backup)) = portable_update_paths() else {
		return;
	};

	for path in [staged, backup] {
		if let Err(err) = fs::remove_file(&path)
			&& err.kind() != std::io::ErrorKind::NotFound
		{
			log::warn!(
				"Failed to remove leftover update file {}: {err}",
				path.display()
			);
		}
	}
}

#[cfg(target_os = "macos")]
fn install_and_restart(bytes: &[u8]) -> Result<(), String> {
	use std::process::Command;

	use rai_pal_backend::files;

	let current_exe =
		app_paths::exe_path().map_err(|err| format!("Could not find the app path: {err}"))?;
	let bundle = current_exe
		.ancestors()
		.find(|path| path.extension().is_some_and(|extension| extension == "app"))
		.ok_or_else(|| "Could not find the app bundle".to_string())?;
	let parent = bundle
		.parent()
		.ok_or_else(|| "Could not find the app bundle folder".to_string())?;

	let update_dir = app_paths::temp_dir("update").map_err(|err| err.to_string())?;
	let archive_path = update_dir.join("updater-macos.tar.gz");
	fs::write(&archive_path, bytes).map_err(|err| err.to_string())?;

	let extract_dir = update_dir.join("extracted");
	if extract_dir.exists() {
		fs::remove_dir_all(&extract_dir).map_err(|err| err.to_string())?;
	}
	files::extract_tar_gz(&archive_path, &extract_dir).map_err(|err| err.to_string())?;

	let new_bundle = fs::read_dir(&extract_dir)
		.map_err(|err| err.to_string())?
		.filter_map(|entry| entry.ok_or_log("Failed to read update directory entry"))
		.map(|entry| entry.path())
		.find(|path| path.extension().is_some_and(|extension| extension == "app"))
		.ok_or_else(|| "Could not find the updated app bundle".to_string())?;

	let bundle_name = bundle
		.file_name()
		.and_then(|name| name.to_str())
		.unwrap_or("RaiPal.app");
	let staged_bundle = parent.join(format!(".{bundle_name}.update"));
	if staged_bundle.exists() {
		fs::remove_dir_all(&staged_bundle).map_err(|err| err.to_string())?;
	}
	crate::runtime::block_on(files::copy_dir_all(&new_bundle, &staged_bundle))?;

	let backup_bundle = parent.join(".RaiPal.app.old");
	if backup_bundle.exists() {
		fs::remove_dir_all(&backup_bundle).map_err(|err| err.to_string())?;
	}
	fs::rename(bundle, &backup_bundle).map_err(|err| err.to_string())?;
	fs::rename(&staged_bundle, bundle).map_err(|err| err.to_string())?;
	fs::remove_dir_all(&backup_bundle).ok();

	let bundle_path = bundle.to_string_lossy().to_string();
	crate::single_instance::release();
	Command::new("open")
		.args(["-n", &bundle_path])
		.spawn()
		.map_err(|err| err.to_string())?;

	std::process::exit(0);
}

#[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
fn install_and_restart(_bytes: &[u8]) -> Result<(), String> {
	Err("Automatic updates are not supported on this platform".to_string())
}

pub fn register(window: &crate::MainWindow) {
	on_command!(window, AppActions, on_install_update, || handler => {
		handler.install_update();
	});
	on_command!(window, AppActions, on_ignore_update, || handler => {
		handler.ignore_update();
	});
}
