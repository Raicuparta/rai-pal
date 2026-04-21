use std::{
	collections::HashMap,
	fs,
	io::{
		Read,
		Write,
	},
	net::TcpListener,
	path::PathBuf,
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
	paths,
	result::{
		Error,
		Result,
	},
};

const DISCORD_AUTH_BASE_URL: &str = "https://discord.com/oauth2/authorize";
const DISCORD_TOKEN_URL: &str = "https://discord.com/api/oauth2/token";

#[derive(Debug, Clone)]
pub struct DiscordOAuthConfig {
	pub client_id: String,
	pub client_secret: Option<String>,
	pub callback_port: u16,
}

#[derive(Clone, Debug, serde::Serialize, specta::Type)]
pub struct DiscordOAuthResult {
	pub token_file_path: String,
	pub token_type: String,
	pub scope: String,
	pub expires_in: u64,
	pub access_token_preview: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct DiscordSavedToken {
	access_token: String,
	token_type: String,
	expires_in: u64,
	refresh_token: Option<String>,
	scope: String,
	received_at_unix_seconds: u64,
}

#[derive(Debug, serde::Deserialize)]
struct DiscordTokenResponse {
	access_token: String,
	token_type: String,
	expires_in: u64,
	refresh_token: Option<String>,
	scope: String,
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
			"<html><body><h2>Discord login complete.</h2><p>You can close this tab and return to Rai Pal.</p></body></html>",
		)
	} else {
		(
			"HTTP/1.1 400 Bad Request",
			"<html><body><h2>Discord login failed.</h2><p>You can close this tab and return to Rai Pal.</p></body></html>",
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

fn parse_oauth_callback(
	listener: TcpListener,
	expected_state: String,
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
					return Err(Error::DiscordOAuth("Malformed callback request.".to_string()));
				};

				let Some(path_and_query) = request_line.split_whitespace().nth(1) else {
					write_browser_response(&mut stream, false)?;
					return Err(Error::DiscordOAuth("Missing callback path.".to_string()));
				};

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

async fn exchange_code_for_discord_token(
	client_id: &str,
	client_secret: Option<&str>,
	code: &str,
	redirect_uri: &str,
	code_verifier: &str,
) -> Result<DiscordTokenResponse> {
	let mut form_data = vec![
		("client_id".to_string(), client_id.to_string()),
		("grant_type".to_string(), "authorization_code".to_string()),
		("code".to_string(), code.to_string()),
		("redirect_uri".to_string(), redirect_uri.to_string()),
		("code_verifier".to_string(), code_verifier.to_string()),
	];

	if let Some(secret) = client_secret {
		form_data.push(("client_secret".to_string(), secret.to_string()));
	}

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

fn save_discord_token_file(token: &DiscordSavedToken) -> Result<PathBuf> {
	let token_file_path = paths::app_data_path()?.join("discord-oauth-token.json");

	if let Some(parent) = token_file_path.parent() {
		fs::create_dir_all(parent)?;
	}

	fs::write(&token_file_path, serde_json::to_vec_pretty(token)?)?;

	Ok(token_file_path)
}

pub async fn start_discord_oauth(config: DiscordOAuthConfig) -> Result<DiscordOAuthResult> {
	let listener = TcpListener::bind(("127.0.0.1", config.callback_port)).map_err(|error| {
		Error::DiscordOAuth(format!(
			"Failed to bind callback port {}. Is another process using it? Error: {error}",
			config.callback_port
		))
	})?;
	let redirect_uri = format!("http://127.0.0.1:{}/discord/callback", config.callback_port);

	let state = create_oauth_nonce();
	let code_verifier = create_oauth_nonce();
	let code_challenge = URL_SAFE_NO_PAD.encode(sha2::Sha256::digest(code_verifier.as_bytes()));

	let auth_url = build_discord_auth_url(
		&config.client_id,
		&redirect_uri,
		&state,
		&code_challenge,
	)?;

	log::info!("Starting Discord OAuth flow. Redirect URI: {redirect_uri}");
	open::that_detached(auth_url)?;

	let auth_code = parse_oauth_callback(listener, state, Duration::from_secs(180))?;

	log::info!("Received Discord OAuth callback. Exchanging code for token...");

	let token_response = exchange_code_for_discord_token(
		&config.client_id,
		config.client_secret.as_deref(),
		&auth_code,
		&redirect_uri,
		&code_verifier,
	)
	.await?;

	let now = SystemTime::now()
		.duration_since(UNIX_EPOCH)
		.map_err(|error| Error::DiscordOAuth(format!("Clock error: {error}")))?
		.as_secs();

	let token_to_save = DiscordSavedToken {
		access_token: token_response.access_token.clone(),
		token_type: token_response.token_type.clone(),
		expires_in: token_response.expires_in,
		refresh_token: token_response.refresh_token.clone(),
		scope: token_response.scope.clone(),
		received_at_unix_seconds: now,
	};

	let token_path = save_discord_token_file(&token_to_save)?;
	log::info!("Saved Discord OAuth token at: {}", token_path.display());

	let preview: String = token_response.access_token.chars().take(8).collect();

	Ok(DiscordOAuthResult {
		token_file_path: token_path.display().to_string(),
		token_type: token_response.token_type,
		scope: token_response.scope,
		expires_in: token_response.expires_in,
		access_token_preview: format!("{preview}..."),
	})
}