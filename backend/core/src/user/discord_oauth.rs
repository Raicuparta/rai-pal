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
		SystemTime,
		UNIX_EPOCH,
	},
};

use base64::{
	Engine,
	engine::general_purpose::URL_SAFE_NO_PAD,
};
use serde::Deserialize;
use sha2::Digest;
use uuid::Uuid;

use crate::{
	http,
	paths::{
		self,
		AsValidStr,
	},
	result::{
		Error,
		Result,
	},
};

const DISCORD_AUTH_BASE_URL: &str = "https://discord.com/oauth2/authorize";
const DISCORD_TOKEN_URL: &str = "https://discord.com/api/oauth2/token";
const DISCORD_USER_URL: &str = "https://discord.com/api/users/@me";
const DISCORD_CLIENT_ID: &str = "1464045413920276694";
const DISCORD_CALLBACK_PORT: u16 = 43941;
const DISCORD_KEYRING_SERVICE: &str = "rai-pal";
const DISCORD_KEYRING_ACCOUNT: &str = "discord-oauth-token";
const DISCORD_KEYRING_LOCATION: &str = "keyring://rai-pal/discord-oauth-token";
const DISCORD_TOKEN_FALLBACK_LOCATION: &str = "file://app_data/discord-oauth-token.json";
const DISCORD_TOKEN_EXPIRY_LEEWAY_SECONDS: u64 = 60;

#[derive(Clone, Debug, serde::Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct DiscordAuthState {
	pub is_logged_in: bool,
	pub avatar_file_path: Option<String>,
	pub user_name: Option<String>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct DiscordSavedToken {
	access_token: String,
	token_type: String,
	expires_in: u64,
	refresh_token: Option<String>,
	scope: String,
	received_at_unix_seconds: u64,
	#[serde(default)]
	user_name: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
struct DiscordTokenResponse {
	access_token: String,
	token_type: String,
	expires_in: u64,
	refresh_token: Option<String>,
	scope: String,
}

#[derive(Debug, serde::Deserialize)]
struct DiscordUserResponse {
	id: String,
	username: String,
	global_name: Option<String>,
	avatar: Option<String>,
}

fn discord_user_display_name(user: &DiscordUserResponse) -> String {
	user.global_name
		.clone()
		.unwrap_or_else(|| user.username.clone())
}

#[derive(Debug, Deserialize)]
struct OAuthCallbackQuery {
	code: Option<String>,
	state: Option<String>,
	error: Option<String>,
}

fn create_oauth_nonce() -> String {
	Uuid::new_v4().simple().to_string()
}

fn create_pkce_code_verifier() -> String {
	// PKCE verifier must be 43..=128 chars and use unreserved URL-safe characters.
	// UUID simple values are [0-9a-f], so concatenating two yields a valid 64-char verifier.
	format!("{}{}", Uuid::new_v4().simple(), Uuid::new_v4().simple())
}

fn build_discord_auth_url(
	client_id: &str,
	redirect_uri: &str,
	state: &str,
	code_challenge: &str,
) -> Result<String> {
	let mut params = HashMap::new();
	params.insert("response_type", "code".to_string());
	params.insert("client_id", client_id.to_string());
	params.insert("scope", "identify".to_string());
	params.insert("redirect_uri", redirect_uri.to_string());
	params.insert("state", state.to_string());
	params.insert("code_challenge", code_challenge.to_string());
	params.insert("code_challenge_method", "S256".to_string());

	let query = serde_urlencoded::to_string(params)?;

	Ok(format!("{DISCORD_AUTH_BASE_URL}?{query}"))
}

fn write_browser_response(stream: &mut std::net::TcpStream, success: bool) -> Result {
	let (status_line, body) = if success {
		(
			"HTTP/1.1 200 OK",
			"<html style=\"background:#fff;\"><body style=\"margin:0;padding:24px;background:#fff;color:#000;\"><h2>Discord login code received.</h2><p>Rai Pal will finish sign-in now. You can close this tab and return to Rai Pal.</p></body></html>",
		)
	} else {
		(
			"HTTP/1.1 400 Bad Request",
			"<html style=\"background:#fff;\"><body style=\"margin:0;padding:24px;background:#fff;color:#000;\"><h2>Discord login failed.</h2><p>You can close this tab and return to Rai Pal.</p></body></html>",
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

fn parse_oauth_callback(
	listener: &TcpListener,
	expected_state: &str,
	timeout: Duration,
) -> Result<String> {
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
					return Err(Error::DiscordOAuth(
						"Malformed callback request.".to_string(),
					));
				};

				let Some(path_and_query) = request_line.split_whitespace().nth(1) else {
					write_browser_response(&mut stream, false)?;
					return Err(Error::DiscordOAuth("Missing callback path.".to_string()));
				};

				if !path_and_query.starts_with("/discord/callback") {
					write_browser_no_content_response(&mut stream)?;
					continue;
				}

				let query = path_and_query
					.split_once('?')
					.map(|(_, query)| query)
					.unwrap_or_default();

				let callback_query = serde_urlencoded::from_str::<OAuthCallbackQuery>(query)
					.map_err(|error| {
						Error::DiscordOAuth(format!("Invalid callback query: {error}"))
					})?;

				if let Some(error) = callback_query.error {
					write_browser_response(&mut stream, false)?;
					return Err(Error::DiscordOAuth(format!(
						"Discord returned OAuth error: {error}"
					)));
				}

				if callback_query.state.is_none() && callback_query.code.is_none() {
					write_browser_no_content_response(&mut stream)?;
					continue;
				}

				let state = callback_query.state.ok_or_else(|| {
					Error::DiscordOAuth("Missing OAuth state in callback.".to_string())
				})?;

				if state != expected_state {
					write_browser_response(&mut stream, false)?;
					return Err(Error::DiscordOAuth("OAuth state mismatch.".to_string()));
				}

				let code = callback_query.code.ok_or_else(|| {
					Error::DiscordOAuth("Missing OAuth code in callback.".to_string())
				})?;

				write_browser_response(&mut stream, true)?;
				return Ok(code);
			}
			Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
				if start.elapsed() >= timeout {
					return Err(Error::DiscordOAuth(
						"Timed out waiting for Discord OAuth callback.".to_string(),
					));
				}

				thread::sleep(Duration::from_millis(100));
			}
			Err(error) => return Err(error.into()),
		}
	}
}

fn get_discord_token_fallback_file_path() -> Result<PathBuf> {
	paths::app_data_file("discord-oauth-token.json")
}

fn read_discord_token_from_fallback_file_optional() -> Result<Option<DiscordSavedToken>> {
	let fallback_path = get_discord_token_fallback_file_path()?;

	if !fallback_path.exists() {
		return Ok(None);
	}

	let token_json = fs::read_to_string(&fallback_path)?;
	let token = serde_json::from_str::<DiscordSavedToken>(&token_json).map_err(|error| {
		Error::DiscordOAuth(format!(
			"Failed to parse Discord token from fallback file `{}`: {error}",
			fallback_path.display()
		))
	})?;

	log::warn!(
		"Using Discord token fallback file storage at `{}`. System keyring appears unavailable.",
		fallback_path.display()
	);

	Ok(Some(token))
}

fn save_discord_token_to_fallback_file(token: &DiscordSavedToken) -> Result<String> {
	let fallback_path = get_discord_token_fallback_file_path()?;

	if let Some(parent) = fallback_path.parent() {
		fs::create_dir_all(parent)?;
	}

	let token_json = serde_json::to_string(token)?;
	fs::write(&fallback_path, token_json)?;

	#[cfg(unix)]
	{
		use std::os::unix::fs::PermissionsExt;
		let mut permissions = fs::metadata(&fallback_path)?.permissions();
		permissions.set_mode(0o600);
		fs::set_permissions(&fallback_path, permissions)?;
	}

	Ok(DISCORD_TOKEN_FALLBACK_LOCATION.to_string())
}

async fn exchange_code_for_discord_token(
	client_id: &str,
	code: &str,
	redirect_uri: &str,
	code_verifier: &str,
) -> Result<DiscordTokenResponse> {
	let form_data = vec![
		("client_id".to_string(), client_id.to_string()),
		("grant_type".to_string(), "authorization_code".to_string()),
		("code".to_string(), code.to_string()),
		("redirect_uri".to_string(), redirect_uri.to_string()),
		("code_verifier".to_string(), code_verifier.to_string()),
	];

	let response = http::CLIENT
		.post(DISCORD_TOKEN_URL)
		.header("content-type", "application/x-www-form-urlencoded")
		.body(serde_urlencoded::to_string(&form_data)?)
		.send()
		.await?;

	if !response.status().is_success() {
		let status = response.status();
		let body = response
			.text()
			.await
			.unwrap_or_else(|_| "<failed to read body>".to_string());

		return Err(Error::DiscordOAuth(format!(
			"Token exchange failed ({status}): {body}"
		)));
	}

	Ok(response.json::<DiscordTokenResponse>().await?)
}

fn get_discord_keyring_entry() -> Result<keyring::Entry> {
	keyring::Entry::new(DISCORD_KEYRING_SERVICE, DISCORD_KEYRING_ACCOUNT).map_err(|error| {
		Error::DiscordOAuth(format!(
			"Failed to open Discord token keyring entry `{DISCORD_KEYRING_LOCATION}`: {error}"
		))
	})
}

fn current_unix_seconds() -> Result<u64> {
	SystemTime::now()
		.duration_since(UNIX_EPOCH)
		.map_err(|error| Error::DiscordOAuth(format!("Clock error: {error}")))
		.map(|duration| duration.as_secs())
}

fn is_discord_token_expired(token: &DiscordSavedToken) -> Result<bool> {
	let expires_at = token
		.received_at_unix_seconds
		.saturating_add(token.expires_in);
	let now_with_leeway =
		current_unix_seconds()?.saturating_add(DISCORD_TOKEN_EXPIRY_LEEWAY_SECONDS);

	Ok(now_with_leeway >= expires_at)
}

fn clear_discord_session() -> Result {
	match get_discord_keyring_entry() {
		Ok(entry) => match entry.delete_credential() {
			Ok(()) | Err(keyring::Error::NoEntry) => {}
			Err(error) => {
				log::warn!(
					"Failed to delete Discord token from system keyring (continuing cleanup): {error}"
				);
			}
		},
		Err(error) => {
			log::warn!(
				"Failed to open Discord token keyring entry while clearing session (continuing cleanup): {error}"
			);
		}
	}

	delete_file_if_exists(&get_discord_token_fallback_file_path()?)?;

	delete_file_if_exists(&get_discord_avatar_file_path()?)?;

	Ok(())
}

fn save_discord_token_file(token: &DiscordSavedToken) -> Result<String> {
	let token_json = serde_json::to_string(token)?;

	match get_discord_keyring_entry() {
		Ok(entry) => match entry.set_password(&token_json) {
			Ok(()) => {
				if let Err(error) = delete_file_if_exists(&get_discord_token_fallback_file_path()?)
				{
					log::warn!(
						"Saved Discord token to keyring but failed to remove fallback file: {error}"
					);
				}
				Ok(DISCORD_KEYRING_LOCATION.to_string())
			}
			Err(error) => {
				log::warn!(
					"Failed to save Discord token in system keyring: {error}. Falling back to file storage."
				);
				save_discord_token_to_fallback_file(token)
			}
		},
		Err(error) => {
			log::warn!(
				"Failed to open Discord keyring entry: {error}. Falling back to file storage."
			);
			save_discord_token_to_fallback_file(token)
		}
	}
}

fn get_discord_avatar_file_path() -> Result<PathBuf> {
	paths::app_data_file("avatar.png")
}

fn delete_file_if_exists(path: &Path) -> Result {
	if path.exists() {
		fs::remove_file(path)?;
	}

	Ok(())
}

async fn fetch_discord_user(access_token: &str) -> Result<DiscordUserResponse> {
	let response = http::CLIENT
		.get(DISCORD_USER_URL)
		.bearer_auth(access_token)
		.send()
		.await?;

	if !response.status().is_success() {
		let status = response.status();
		let body = response
			.text()
			.await
			.unwrap_or_else(|_| "<failed to read body>".to_string());

		return Err(Error::DiscordOAuth(format!(
			"Failed to fetch Discord user profile ({status}): {body}"
		)));
	}

	Ok(response.json::<DiscordUserResponse>().await?)
}

async fn download_and_save_discord_avatar(
	access_token: &str,
	user: &DiscordUserResponse,
) -> Result<Option<String>> {
	let avatar_file_path = get_discord_avatar_file_path()?;

	let Some(avatar_hash) = user.avatar.as_ref() else {
		delete_file_if_exists(&avatar_file_path)?;
		return Ok(None);
	};

	let avatar_url = format!(
		"https://cdn.discordapp.com/avatars/{}/{avatar_hash}.png?size=128",
		user.id
	);

	let response = http::CLIENT
		.get(avatar_url)
		.bearer_auth(access_token)
		.send()
		.await?;

	if !response.status().is_success() {
		let status = response.status();
		let body = response
			.text()
			.await
			.unwrap_or_else(|_| "<failed to read body>".to_string());

		return Err(Error::DiscordOAuth(format!(
			"Failed to download Discord avatar ({status}): {body}"
		)));
	}

	if let Some(parent) = avatar_file_path.parent() {
		fs::create_dir_all(parent)?;
	}

	fs::write(&avatar_file_path, response.bytes().await?)?;

	Ok(Some(avatar_file_path.try_to_str()?.to_string()))
}

fn read_discord_token_file_optional() -> Result<Option<DiscordSavedToken>> {
	let entry = match get_discord_keyring_entry() {
		Ok(entry) => entry,
		Err(error) => {
			log::warn!(
				"Failed to open Discord keyring entry: {error}. Trying fallback file storage."
			);
			return read_discord_token_from_fallback_file_optional();
		}
	};

	match entry.get_password() {
		Ok(token_json) => {
			let token =
				serde_json::from_str::<DiscordSavedToken>(&token_json).map_err(|error| {
					Error::DiscordOAuth(format!(
						"Failed to parse Discord token from system keyring: {error}"
					))
				})?;
			let access_token_preview: String = token.access_token.chars().take(8).collect();
			log::debug!(
				"Read Discord token from keyring; user_name={:?} access_token_preview={}...",
				token.user_name,
				access_token_preview
			);
			Ok(Some(token))
		}
		Err(keyring::Error::NoEntry) => {
			log::debug!(
				"No Discord token found in keyring ({DISCORD_KEYRING_LOCATION}); checking fallback file."
			);
			read_discord_token_from_fallback_file_optional()
		}
		Err(error) => {
			log::warn!(
				"Failed to read Discord token from system keyring: {error}. Trying fallback file storage."
			);
			read_discord_token_from_fallback_file_optional()
		}
	}
}

fn read_discord_token_file() -> Result<DiscordSavedToken> {
	read_discord_token_file_optional()?
		.ok_or_else(|| Error::DiscordOAuth("Discord OAuth token is not available.".to_string()))
}

async fn exchange_refresh_token_for_discord_token(
	client_id: &str,
	refresh_token: &str,
) -> Result<DiscordTokenResponse> {
	let form_data = vec![
		("client_id".to_string(), client_id.to_string()),
		("grant_type".to_string(), "refresh_token".to_string()),
		("refresh_token".to_string(), refresh_token.to_string()),
	];

	let response = http::CLIENT
		.post(DISCORD_TOKEN_URL)
		.header("content-type", "application/x-www-form-urlencoded")
		.body(serde_urlencoded::to_string(&form_data)?)
		.send()
		.await?;

	if !response.status().is_success() {
		let status = response.status();
		let body = response
			.text()
			.await
			.unwrap_or_else(|_| "<failed to read body>".to_string());

		return Err(Error::DiscordOAuth(format!(
			"Token refresh failed ({status}): {body}"
		)));
	}

	Ok(response.json::<DiscordTokenResponse>().await?)
}

pub async fn refresh_discord_token_if_possible() -> Result<bool> {
	let Some(saved_token) = read_discord_token_file_optional()? else {
		log::debug!("Skipping Discord token refresh: Discord token not found in system keyring");
		return Ok(false);
	};

	let Some(refresh_token) = saved_token.refresh_token else {
		log::debug!("Skipping Discord token refresh: saved token has no refresh_token");
		return Ok(false);
	};

	let token_response =
		exchange_refresh_token_for_discord_token(DISCORD_CLIENT_ID, &refresh_token).await?;

	let now = current_unix_seconds()?;

	let refreshed_token = DiscordSavedToken {
		access_token: token_response.access_token,
		token_type: token_response.token_type,
		expires_in: token_response.expires_in,
		refresh_token: token_response.refresh_token.or(Some(refresh_token)),
		scope: token_response.scope,
		received_at_unix_seconds: now,
		user_name: saved_token.user_name,
	};

	save_discord_token_file(&refreshed_token)?;

	Ok(true)
}

pub async fn get_discord_auth_state() -> Result<DiscordAuthState> {
	log::debug!("Computing Discord auth state");
	let Some(mut saved_token) = read_discord_token_file_optional()? else {
		log::debug!("Discord auth state: logged out (no token in keyring)");
		return Ok(DiscordAuthState {
			is_logged_in: false,
			avatar_file_path: None,
			user_name: None,
		});
	};

	if is_discord_token_expired(&saved_token)? {
		log::info!("Discord auth token is expired; attempting immediate refresh.");

		match refresh_discord_token_if_possible().await {
			Ok(true) => {
				log::info!("Discord auth token refreshed while computing auth state.");
				if let Some(refreshed_token) = read_discord_token_file_optional()? {
					saved_token = refreshed_token;
				} else {
					log::warn!(
						"Discord token disappeared after refresh. Clearing session and marking logged out."
					);
					clear_discord_session()?;
					return Ok(DiscordAuthState {
						is_logged_in: false,
						avatar_file_path: None,
						user_name: None,
					});
				}
			}
			Ok(false) => {
				log::info!(
					"Discord auth token expired and refresh is unavailable. Clearing session and marking logged out."
				);
				clear_discord_session()?;
				return Ok(DiscordAuthState {
					is_logged_in: false,
					avatar_file_path: None,
					user_name: None,
				});
			}
			Err(error) => {
				log::warn!(
					"Discord auth token expired and immediate refresh failed: {error}. Clearing session and marking logged out."
				);
				clear_discord_session()?;
				return Ok(DiscordAuthState {
					is_logged_in: false,
					avatar_file_path: None,
					user_name: None,
				});
			}
		}

		if is_discord_token_expired(&saved_token)? {
			log::warn!(
				"Discord token is still expired after refresh attempt. Clearing session and marking logged out."
			);
			clear_discord_session()?;
			return Ok(DiscordAuthState {
				is_logged_in: false,
				avatar_file_path: None,
				user_name: None,
			});
		}
	}

	let avatar_file_path = get_discord_avatar_file_path()?;

	Ok(DiscordAuthState {
		is_logged_in: true,
		avatar_file_path: Some(avatar_file_path.try_to_str()?.to_string()),
		user_name: saved_token.user_name,
	})
}

pub(crate) fn read_discord_access_token() -> Result<String> {
	let saved_token = read_discord_token_file()?;

	if is_discord_token_expired(&saved_token)? {
		return Err(Error::DiscordOAuth(
			"Discord OAuth token has expired. Please sign in again.".to_string(),
		));
	}

	Ok(saved_token.access_token)
}

pub fn logout_discord() -> Result {
	clear_discord_session()
}

pub async fn start_discord_oauth() -> Result {
	let listener = TcpListener::bind(("127.0.0.1", DISCORD_CALLBACK_PORT)).map_err(|error| {
		Error::DiscordOAuth(format!(
			"Failed to bind callback port {DISCORD_CALLBACK_PORT}. Is another process using it? Error: {error}"
		))
	})?;
	let redirect_uri = format!("http://127.0.0.1:{DISCORD_CALLBACK_PORT}/discord/callback");

	let state = create_oauth_nonce();
	let code_verifier = create_pkce_code_verifier();
	let code_challenge = URL_SAFE_NO_PAD.encode(sha2::Sha256::digest(code_verifier.as_bytes()));

	let auth_url =
		build_discord_auth_url(DISCORD_CLIENT_ID, &redirect_uri, &state, &code_challenge)?;

	log::info!("Starting Discord OAuth flow. Redirect URI: {redirect_uri}");
	open::that_detached(auth_url)?;

	let auth_code = parse_oauth_callback(&listener, &state, Duration::from_mins(3))?;

	log::info!("Received Discord OAuth callback. Exchanging code for token...");

	let token_response = exchange_code_for_discord_token(
		DISCORD_CLIENT_ID,
		&auth_code,
		&redirect_uri,
		&code_verifier,
	)
	.await?;

	let now = current_unix_seconds()?;

	let mut token_to_save = DiscordSavedToken {
		access_token: token_response.access_token.clone(),
		token_type: token_response.token_type.clone(),
		expires_in: token_response.expires_in,
		refresh_token: token_response.refresh_token.clone(),
		scope: token_response.scope.clone(),
		received_at_unix_seconds: now,
		user_name: None,
	};

	let user = fetch_discord_user(&token_response.access_token).await?;
	token_to_save.user_name = Some(discord_user_display_name(&user));

	let token_path = save_discord_token_file(&token_to_save)?;
	log::info!("Saved Discord OAuth token at: {token_path}");

	let avatar_path = download_and_save_discord_avatar(&token_response.access_token, &user).await?;
	if let Some(path) = avatar_path {
		log::info!("Saved Discord avatar at: {path}");
	}

	Ok(())
}
