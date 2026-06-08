use std::{
	collections::HashMap,
	fs,
	io::{
		Read,
		Write,
	},
	net::TcpListener,
	path::{
		Path,
		PathBuf,
	},
	thread,
	time::{
		Duration,
		Instant,
	},
};

use serde::Deserialize;
use uuid::Uuid;

use crate::{
	app_paths,
	http,
	open_better::open_detached_better,
	result::{
		Error,
		Result,
	},
};

const AUTH_URL_BASE: &str = "https://auth.raicuparta.com";
const AUTH_KEYRING_SERVICE: &str = "rai-pal";
const AUTH_KEYRING_ACCOUNT: &str = "auth-session-token";
const AUTH_KEYRING_LOCATION: &str = "keyring://rai-pal/auth-session-token";
const AUTH_SESSION_FALLBACK_LOCATION: &str = "file://app_data/auth-session.json";

#[derive(Clone, Debug, serde::Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct AuthState {
	pub is_logged_in: bool,
	pub avatar_url: Option<String>,
	pub user_name: Option<String>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct AuthSavedSession {
	token: String,
	user_name: String,
	#[serde(default)]
	avatar_url: Option<String>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct AuthSessionResponse {
	user_name: String,
	avatar_url: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AuthCallbackQuery {
	token: Option<String>,
	state: Option<String>,
	error: Option<String>,
}

fn create_oauth_nonce() -> String {
	Uuid::new_v4().simple().to_string()
}

fn build_auth_start_url(start_url: &str, redirect_uri: &str, state: &str) -> Result<String> {
	let mut params = HashMap::new();
	params.insert("redirect_uri".to_string(), redirect_uri.to_string());
	params.insert("state".to_string(), state.to_string());

	let query = serde_urlencoded::to_string(params)?;

	Ok(format!("{start_url}?{query}"))
}

fn write_browser_response(stream: &mut std::net::TcpStream, success: bool) -> Result {
	let (status_line, body) = if success {
		(
			"HTTP/1.1 200 OK",
			"<html style=\"background:#fff;\"><body style=\"margin:0;padding:24px;background:#fff;color:#000;\"><h2>Authentication complete.</h2><p>You can close this tab and return to Rai Pal.</p></body></html>",
		)
	} else {
		(
			"HTTP/1.1 400 Bad Request",
			"<html style=\"background:#fff;\"><body style=\"margin:0;padding:24px;background:#fff;color:#000;\"><h2>Authentication failed.</h2><p>You can close this tab and return to Rai Pal.</p></body></html>",
		)
	};

	let response = format!(
		"{status_line}\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
		body.len(),
		body
	);

	stream.write_all(response.as_bytes())?;
	stream.flush()?;

	Ok(())
}

fn write_browser_no_content_response(stream: &mut std::net::TcpStream) -> Result {
	let response = "HTTP/1.1 204 No Content\r\nContent-Length: 0\r\nConnection: close\r\n\r\n";

	stream.write_all(response.as_bytes())?;
	stream.flush()?;

	Ok(())
}

fn parse_auth_callback(
	listener: &TcpListener,
	expected_state: &str,
	timeout: Duration,
) -> Result<AuthCallbackQuery> {
	listener.set_nonblocking(true)?;
	let start = Instant::now();

	loop {
		match listener.accept() {
			Ok((mut stream, _addr)) => {
				let mut buffer = [0_u8; 4096];
				let bytes_read = stream.read(&mut buffer)?;

				if bytes_read == 0 {
					continue;
				}

				let request = String::from_utf8_lossy(&buffer[..bytes_read]);
				let Some(request_line) = request.lines().next() else {
					write_browser_response(&mut stream, false)?;
					return Err(Error::Auth("Malformed callback request.".to_string()));
				};

				let Some(path_and_query) = request_line.split_whitespace().nth(1) else {
					write_browser_response(&mut stream, false)?;
					return Err(Error::Auth("Missing callback path.".to_string()));
				};

				if !path_and_query.starts_with("/auth/callback") {
					write_browser_no_content_response(&mut stream)?;
					continue;
				}

				log::info!("callback query: {}", path_and_query);

				let query = path_and_query
					.split_once('?')
					.map(|(_, query)| query)
					.unwrap_or_default();

				// log query for debeug

				let callback_query = serde_urlencoded::from_str::<AuthCallbackQuery>(query)
					.map_err(|error| Error::Auth(format!("Invalid callback query: {error}")))?;

				if let Some(error) = callback_query.error.as_ref() {
					write_browser_response(&mut stream, false)?;
					return Err(Error::Auth(format!(
						"Authentication server returned an error: {error}"
					)));
				}

				if callback_query.state.is_none() && callback_query.token.is_none() {
					write_browser_no_content_response(&mut stream)?;
					continue;
				}

				let state = callback_query
					.state
					.ok_or_else(|| Error::Auth("Missing auth state in callback.".to_string()))?;

				if state != expected_state {
					write_browser_response(&mut stream, false)?;
					return Err(Error::Auth("Auth state mismatch.".to_string()));
				}

				let token = callback_query
					.token
					.ok_or_else(|| Error::Auth("Missing auth token in callback.".to_string()))?;

				write_browser_response(&mut stream, true)?;
				return Ok(AuthCallbackQuery {
					token: Some(token),
					state: Some(state),
					error: None,
				});
			}
			Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
				if start.elapsed() >= timeout {
					return Err(Error::Auth(
						"Timed out waiting for auth callback.".to_string(),
					));
				}

				thread::sleep(Duration::from_millis(100));
			}
			Err(error) => return Err(error.into()),
		}
	}
}

fn get_auth_session_fallback_file_path() -> Result<PathBuf> {
	app_paths::app_data_file("auth-session.json")
}

fn read_auth_session_from_fallback_file_optional() -> Result<Option<AuthSavedSession>> {
	let fallback_path = get_auth_session_fallback_file_path()?;

	if !fallback_path.exists() {
		return Ok(None);
	}

	let session_json = fs::read_to_string(&fallback_path)?;
	let session = serde_json::from_str::<AuthSavedSession>(&session_json).map_err(|error| {
		Error::Auth(format!(
			"Failed to parse auth session from fallback file `{}`: {error}",
			fallback_path.display()
		))
	})?;

	log::warn!(
		"Using auth session fallback file storage at `{}`. System keyring appears unavailable.",
		fallback_path.display()
	);

	Ok(Some(session))
}

fn save_auth_session_to_fallback_file(session: &AuthSavedSession) -> Result<String> {
	let fallback_path = get_auth_session_fallback_file_path()?;

	if let Some(parent) = fallback_path.parent() {
		fs::create_dir_all(parent)?;
	}

	let session_json = serde_json::to_string(session)?;
	fs::write(&fallback_path, session_json)?;

	#[cfg(unix)]
	{
		use std::os::unix::fs::PermissionsExt;
		let mut permissions = fs::metadata(&fallback_path)?.permissions();
		permissions.set_mode(0o600);
		fs::set_permissions(&fallback_path, permissions)?;
	}

	Ok(AUTH_SESSION_FALLBACK_LOCATION.to_string())
}

fn get_auth_keyring_entry() -> Result<keyring::Entry> {
	keyring::Entry::new(AUTH_KEYRING_SERVICE, AUTH_KEYRING_ACCOUNT).map_err(|error| {
		Error::Auth(format!(
			"Failed to open auth session keyring entry `{AUTH_KEYRING_LOCATION}`: {error}"
		))
	})
}

fn clear_auth_session() -> Result {
	match get_auth_keyring_entry() {
		Ok(entry) => match entry.delete_credential() {
			Ok(()) | Err(keyring::Error::NoEntry) => {}
			Err(error) => {
				log::warn!(
					"Failed to delete auth session from system keyring (continuing cleanup): {error}"
				);
			}
		},
		Err(error) => {
			log::warn!(
				"Failed to open auth session keyring entry while clearing session (continuing cleanup): {error}"
			);
		}
	}

	delete_file_if_exists(&get_auth_session_fallback_file_path()?)?;

	Ok(())
}

fn save_auth_session_file(session: &AuthSavedSession) -> Result<String> {
	let session_json = serde_json::to_string(session)?;

	match get_auth_keyring_entry() {
		Ok(entry) => match entry.set_password(&session_json) {
			Ok(()) => {
				if let Err(error) = delete_file_if_exists(&get_auth_session_fallback_file_path()?) {
					log::warn!(
						"Saved auth session to keyring but failed to remove fallback file: {error}"
					);
				}
				Ok(AUTH_KEYRING_LOCATION.to_string())
			}
			Err(error) => {
				log::warn!(
					"Failed to save auth session in system keyring: {error}. Falling back to file storage."
				);
				save_auth_session_to_fallback_file(session)
			}
		},
		Err(error) => {
			log::warn!("Failed to open auth keyring entry: {error}. Falling back to file storage.");
			save_auth_session_to_fallback_file(session)
		}
	}
}

fn delete_file_if_exists(path: &Path) -> Result {
	if path.exists() {
		fs::remove_file(path)?;
	}

	Ok(())
}

fn read_auth_session_file_optional() -> Result<Option<AuthSavedSession>> {
	let entry = match get_auth_keyring_entry() {
		Ok(entry) => entry,
		Err(error) => {
			log::warn!("Failed to open auth keyring entry: {error}. Trying fallback file storage.");
			return read_auth_session_from_fallback_file_optional();
		}
	};

	match entry.get_password() {
		Ok(session_json) => {
			let session =
				serde_json::from_str::<AuthSavedSession>(&session_json).map_err(|error| {
					Error::Auth(format!(
						"Failed to parse auth session from system keyring: {error}"
					))
				})?;
			let token_preview: String = session.token.chars().take(8).collect();
			log::debug!(
				"Read auth session from keyring; user_name={:?} token_preview={}...",
				session.user_name,
				token_preview
			);
			Ok(Some(session))
		}
		Err(keyring::Error::NoEntry) => {
			log::debug!(
				"No auth session found in keyring ({AUTH_KEYRING_LOCATION}); checking fallback file."
			);
			read_auth_session_from_fallback_file_optional()
		}
		Err(error) => {
			log::warn!(
				"Failed to read auth session from system keyring: {error}. Trying fallback file storage."
			);
			read_auth_session_from_fallback_file_optional()
		}
	}
}

fn read_auth_session_file() -> Result<AuthSavedSession> {
	read_auth_session_file_optional()?
		.ok_or_else(|| Error::Auth("Auth token is not available.".to_string()))
}

pub fn get_user_auth_state() -> Result<AuthState> {
	log::debug!("Computing auth state");
	let Some(saved_session) = read_auth_session_file_optional()? else {
		log::debug!("Auth state: logged out (no session found)");
		return Ok(AuthState {
			is_logged_in: false,
			avatar_url: None,
			user_name: None,
		});
	};

	Ok(AuthState {
		is_logged_in: true,
		avatar_url: saved_session.avatar_url,
		user_name: Some(saved_session.user_name),
	})
}

pub(crate) fn read_auth_token() -> Result<String> {
	let saved_session = read_auth_session_file()?;

	Ok(saved_session.token)
}

pub fn logout_auth() -> Result {
	clear_auth_session()
}

pub async fn start_auth() -> Result {
	let listener = TcpListener::bind(("127.0.0.1", 0))
		.map_err(|error| Error::Auth(format!("Failed to bind callback port. Error: {error}")))?;
	let local_address = listener.local_addr()?;
	let redirect_uri = format!("http://{local_address}/auth/callback");

	let state = create_oauth_nonce();
	let auth_url = build_auth_start_url(&format!("{AUTH_URL_BASE}/start"), &redirect_uri, &state)?;

	log::info!("Starting auth flow. Redirect URI: {redirect_uri}");
	open_detached_better(auth_url)?;

	let callback = parse_auth_callback(&listener, &state, Duration::from_mins(3))?;

	log::info!("Received auth callback. Getting session...");

	let auth_token = callback
		.token
		.ok_or_else(|| Error::Auth("Missing auth token in callback.".to_string()))?;

	let result = http::CLIENT
		.get(format!("{AUTH_URL_BASE}/user"))
		.header("Authorization", format!("Bearer {auth_token}"))
		.send()
		.await?;

	let session = result.json::<AuthSessionResponse>().await?;

	let session = AuthSavedSession {
		token: auth_token,
		user_name: session.user_name,
		avatar_url: session.avatar_url,
	};

	let session_path = save_auth_session_file(&session)?;
	log::info!("Saved auth session at: {session_path}");

	Ok(())
}
