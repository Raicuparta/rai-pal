use std::{
	collections::HashMap,
	path::PathBuf,
	time::Duration,
};

use rai_pal_proc_macros::serializable_struct;
use serde::Deserialize;
use tokio::{
	io::{
		AsyncReadExt,
		AsyncWriteExt,
	},
	net::{
		TcpListener,
		TcpStream,
	},
	time::timeout,
};
use uuid::Uuid;

use crate::{
	app_paths,
	http,
	open_better::open_detached_better,
	path_extensions::AsValidStr,
	result::{
		Error,
		LogErrExt,
		Result,
	},
};

const AUTH_URL_BASE: &str = "https://auth.raicuparta.com";
const AUTH_KEYRING_SERVICE: &str = "rai-pal";
const AUTH_KEYRING_ACCOUNT: &str = "auth-session-token";

#[derive(Clone, Debug, serde::Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct AuthState {
	pub is_logged_in: bool,
	pub avatar_path: Option<String>,
	pub user_name: Option<String>,
}

#[serializable_struct]
struct AuthSavedSession {
	token: String,
	user_name: String,
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

async fn write_redirect(stream: &mut TcpStream, success: bool) -> Result {
	let path = if success { "success" } else { "error" };
	let response = format!(
		"HTTP/1.1 302 Found\r\nLocation: {AUTH_URL_BASE}/{path}\r\nConnection: close\r\n\r\n"
	);
	stream.write_all(response.as_bytes()).await?;
	stream.flush().await?;
	Ok(())
}

async fn write_no_content(stream: &mut TcpStream) -> Result {
	let response = "HTTP/1.1 204 No Content\r\nContent-Length: 0\r\nConnection: close\r\n\r\n";
	stream.write_all(response.as_bytes()).await?;
	stream.flush().await?;
	Ok(())
}

async fn parse_auth_callback(
	listener: &TcpListener,
	expected_state: &str,
) -> Result<AuthCallbackQuery> {
	loop {
		let (mut stream, _addr) = listener.accept().await?;

		let mut buffer = [0_u8; 4096];
		let bytes_read = stream.read(&mut buffer).await?;

		if bytes_read == 0 {
			continue;
		}

		let request = String::from_utf8_lossy(&buffer[..bytes_read]);
		let Some(request_line) = request.lines().next() else {
			write_redirect(&mut stream, false).await?;
			return Err(Error::Auth("Malformed callback request.".to_string()));
		};

		let Some(path_and_query) = request_line.split_whitespace().nth(1) else {
			write_redirect(&mut stream, false).await?;
			return Err(Error::Auth("Missing callback path.".to_string()));
		};

		if !path_and_query.starts_with("/auth/callback") {
			write_no_content(&mut stream).await?;
			continue;
		}

		let query = path_and_query
			.split_once('?')
			.map(|(_, query)| query)
			.unwrap_or_default();

		let callback_query = serde_urlencoded::from_str::<AuthCallbackQuery>(query)
			.map_err(|error| Error::Auth(format!("Invalid callback query: {error}")))?;

		if let Some(error) = callback_query.error.as_ref() {
			write_redirect(&mut stream, false).await?;
			return Err(Error::Auth(format!(
				"Authentication server returned an error: {error}"
			)));
		}

		if callback_query.state.is_none() && callback_query.token.is_none() {
			write_no_content(&mut stream).await?;
			continue;
		}

		let state = callback_query
			.state
			.as_deref()
			.ok_or_else(|| Error::Auth("Missing auth state in callback.".to_string()))?;

		if state != expected_state {
			write_redirect(&mut stream, false).await?;
			return Err(Error::Auth("Auth state mismatch.".to_string()));
		}

		let token = callback_query
			.token
			.clone()
			.ok_or_else(|| Error::Auth("Missing auth token in callback.".to_string()))?;

		write_redirect(&mut stream, true).await?;
		return Ok(AuthCallbackQuery {
			token: Some(token),
			state: Some(state.to_string()),
			error: None,
		});
	}
}

fn get_auth_keyring_entry() -> Result<keyring::Entry> {
	keyring::Entry::new(AUTH_KEYRING_SERVICE, AUTH_KEYRING_ACCOUNT).map_err(|error| {
		Error::Auth(format!(
			"Failed to open auth session keyring entry: {error}"
		))
	})
}

pub async fn logout_auth() -> Result {
	if let Ok(entry) = get_auth_keyring_entry() {
		let _ = entry.delete_credential();
	}

	let fallback_path = app_paths::app_data_file("auth-session.json")?;
	if fallback_path.exists() {
		tokio::fs::remove_file(fallback_path).await?;
	}

	Ok(())
}

async fn save_auth_session_file(session: &AuthSavedSession) -> Result {
	let session_json = serde_json::to_string(session)?;
	let fallback_path = app_paths::app_data_file("auth-session.json")?;

	// Try storing in the secure keyring first
	if let Ok(entry) = get_auth_keyring_entry()
		&& entry.set_password(&session_json).is_ok()
	{
		let _ = tokio::fs::remove_file(&fallback_path).await;
		return Ok(());
	}

	// Keyring not available or failed; write to fallback file
	if let Some(parent) = fallback_path.parent() {
		tokio::fs::create_dir_all(parent).await?;
	}
	tokio::fs::write(&fallback_path, &session_json).await?;

	#[cfg(unix)]
	{
		use std::os::unix::fs::PermissionsExt;
		if let Ok(metadata) = tokio::fs::metadata(&fallback_path).await {
			let mut permissions = metadata.permissions();
			permissions.set_mode(0o600);
			let _ = tokio::fs::set_permissions(&fallback_path, permissions).await;
		}
	}

	Ok(())
}

fn read_auth_session_file_optional() -> Result<Option<AuthSavedSession>> {
	// Try reading from keyring
	if let Ok(entry) = get_auth_keyring_entry()
		&& let Ok(session_json) = entry.get_password()
	{
		let session = serde_json::from_str::<AuthSavedSession>(&session_json).map_err(|error| {
			Error::Auth(format!(
				"Failed to parse auth session from keyring: {error}"
			))
		})?;
		return Ok(Some(session));
	}

	// Try fallback file
	let fallback_path = app_paths::app_data_file("auth-session.json")?;
	if !fallback_path.exists() {
		return Ok(None);
	}

	let session_json = std::fs::read_to_string(&fallback_path)?;
	let session = serde_json::from_str::<AuthSavedSession>(&session_json).map_err(|error| {
		Error::Auth(format!(
			"Failed to parse auth session from file `{}`: {error}",
			fallback_path.display()
		))
	})?;

	Ok(Some(session))
}

pub(crate) fn read_auth_token() -> Result<String> {
	let saved_session = read_auth_session_file_optional()?
		.ok_or_else(|| Error::Auth("Auth token is not available.".to_string()))?;
	Ok(saved_session.token)
}

async fn refresh_auth(auth_token: &str) -> Result<AuthState> {
	let result = http::CLIENT
		.get(format!("{AUTH_URL_BASE}/user"))
		.header("Authorization", format!("Bearer {auth_token}"))
		.send()
		.await?;

	if !result.status().is_success() {
		return Ok(AuthState {
			is_logged_in: false,
			avatar_path: None,
			user_name: None,
		});
	}

	let session_response = result.json::<AuthSessionResponse>().await?;
	let session = AuthSavedSession {
		token: auth_token.to_string(),
		user_name: session_response.user_name,
		avatar_url: session_response.avatar_url,
	};

	save_auth_session_file(&session).await?;

	let avatar_path = if let Some(avatar_url) = session.avatar_url {
		save_avatar(&avatar_url)
			.await
			.ok_or_log("Failed to get user avatar")
			.flatten()
	} else {
		None
	};

	Ok(AuthState {
		is_logged_in: true,
		avatar_path,
		user_name: Some(session.user_name),
	})
}

fn get_avatar_path() -> Result<PathBuf> {
	app_paths::app_data_file("avatar.png")
}

async fn save_avatar(url: &str) -> Result<Option<String>> {
	let avatar_path = get_avatar_path()?;

	let response = http::CLIENT.get(url).send().await?;

	tokio::fs::write(&avatar_path, response.bytes().await?).await?;

	Ok(Some(avatar_path.try_to_str()?.to_string()))
}

pub async fn get_user_auth_state() -> Result<AuthState> {
	let Some(saved_session) = read_auth_session_file_optional()? else {
		return Ok(AuthState {
			is_logged_in: false,
			avatar_path: None,
			user_name: None,
		});
	};

	refresh_auth(&saved_session.token).await
}

pub async fn start_auth() -> Result {
	let listener = TcpListener::bind(("127.0.0.1", 0))
		.await
		.map_err(|error| Error::Auth(format!("Failed to bind callback port: {error}")))?;

	let redirect_uri = format!("http://{}/auth/callback", listener.local_addr()?);
	let state = Uuid::new_v4().simple().to_string();

	let mut params = HashMap::new();
	params.insert("redirect_uri", &redirect_uri);
	params.insert("state", &state);
	let query = serde_urlencoded::to_string(params)?;
	let auth_url = format!("{AUTH_URL_BASE}/start?{query}");

	open_detached_better(auth_url)?;

	let callback = timeout(
		Duration::from_mins(3),
		parse_auth_callback(&listener, &state),
	)
	.await
	.map_err(|_| Error::Auth("Timed out waiting for auth callback.".to_string()))??;

	let token = callback
		.token
		.ok_or_else(|| Error::Auth("Missing auth token in callback.".to_string()))?;

	refresh_auth(&token).await?;

	Ok(())
}
