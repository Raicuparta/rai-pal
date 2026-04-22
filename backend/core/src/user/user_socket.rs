use std::{
	fs,
	io::{
		Read,
		Write,
	},
	net::{
		TcpListener,
		TcpStream,
	},
	path::PathBuf,
	thread,
	time::Duration,
};

use serde::Deserialize;

use crate::{
	paths,
	result::{
		Error,
		Result,
	},
};

const USER_SOCKET_BIND_ADDRESS: &str = "127.0.0.1";
const USER_SOCKET_PORT_RANGE_START: u16 = 43950;
const USER_SOCKET_PORT_RANGE_END: u16 = 43960;
const USER_SOCKET_POLL_INTERVAL: Duration = Duration::from_millis(100);

#[derive(Debug, Deserialize)]
struct DiscordSavedToken {
	access_token: String,
}

pub fn start_user_socket_manager() {
	thread::spawn(|| {
		let mut listener: Option<TcpListener> = None;
		let mut bind_error_logged = false;

		loop {
			let is_logged_in = is_discord_user_logged_in();

			if is_logged_in && listener.is_none() {
				match bind_first_available_port() {
					Ok((new_listener, new_port)) => {
						if let Err(error) = new_listener.set_nonblocking(true) {
							log::error!(
								"Failed to set user socket listener to non-blocking mode: {error}"
							);
						} else {
							log::info!(
								"User socket server is listening at {USER_SOCKET_BIND_ADDRESS}:{new_port}"
							);
							listener = Some(new_listener);
							bind_error_logged = false;
						}
					}
					Err(error) => {
						if !bind_error_logged {
							log::error!("Failed to start user socket server: {error}");
							bind_error_logged = true;
						}
					}
				}
			}

			if !is_logged_in && listener.is_some() {
				log::info!("User logged out. Stopping user socket server.");
				listener = None;
			}

			if let Some(active_listener) = listener.as_ref() {
				match active_listener.accept() {
					Ok((mut stream, _)) => {
						if let Err(error) = handle_socket_connection(&mut stream) {
							log::error!("Failed to handle user socket request: {error}");
						}
					}
					Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {}
					Err(error) => {
						log::error!("User socket accept failed: {error}");
						listener = None;
					}
				}
			}

			thread::sleep(USER_SOCKET_POLL_INTERVAL);
		}
	});
}

fn handle_socket_connection(stream: &mut TcpStream) -> Result {
	let mut buffer = [0_u8; 4096];
	let bytes_read = stream.read(&mut buffer)?;

	if bytes_read == 0 {
		return Ok(());
	}

	let request = String::from_utf8_lossy(&buffer[..bytes_read]);
	let Some(request_line) = request.lines().next() else {
		write_http_response(stream, 400, "Bad Request", "Malformed request")?;
		return Ok(());
	};

	let mut line_parts = request_line.split_whitespace();
	let method = line_parts.next().unwrap_or_default();
	let path = line_parts.next().unwrap_or_default();

	if method != "GET" {
		write_http_response(stream, 405, "Method Not Allowed", "Only GET is supported")?;
		return Ok(());
	}

	if path != "/token" {
		write_http_response(stream, 404, "Not Found", "Unknown path")?;
		return Ok(());
	}

	match read_discord_access_token() {
		Ok(access_token) => {
			write_http_response(stream, 200, "OK", &access_token)?;
		}
		Err(error) => {
			write_http_response(stream, 401, "Unauthorized", "Discord user is not logged in")?;
			log::debug!("Unable to serve /token because token is unavailable: {error}");
		}
	}

	Ok(())
}

fn write_http_response(
	stream: &mut TcpStream,
	status_code: u16,
	status_text: &str,
	body: &str,
) -> Result {
	let response = format!(
		"HTTP/1.1 {status_code} {status_text}\r\nContent-Type: text/plain; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
		body.len(),
		body
	);

	stream.write_all(response.as_bytes())?;
	stream.flush()?;

	Ok(())
}

fn bind_first_available_port() -> Result<(TcpListener, u16)> {
	for port in USER_SOCKET_PORT_RANGE_START..=USER_SOCKET_PORT_RANGE_END {
		match TcpListener::bind((USER_SOCKET_BIND_ADDRESS, port)) {
			Ok(listener) => return Ok((listener, port)),
			Err(error) if error.kind() == std::io::ErrorKind::AddrInUse => {}
			Err(error) => {
				return Err(Error::DiscordOAuth(format!(
					"Failed to bind user socket at {USER_SOCKET_BIND_ADDRESS}:{port}: {error}"
				)));
			}
		}
	}

	Err(Error::DiscordOAuth(format!(
		"No available user socket ports in range {USER_SOCKET_PORT_RANGE_START}..={USER_SOCKET_PORT_RANGE_END}"
	)))
}

fn read_discord_access_token() -> Result<String> {
	let token_file_path = get_user_file_path()?;
	let token_contents = fs::read_to_string(&token_file_path).map_err(|error| {
		Error::DiscordOAuth(format!(
			"Failed to read Discord token file `{}`: {error}",
			token_file_path.display()
		))
	})?;

	let token = serde_json::from_str::<DiscordSavedToken>(&token_contents).map_err(|error| {
		Error::DiscordOAuth(format!(
			"Failed to parse Discord token file `{}`: {error}",
			token_file_path.display()
		))
	})?;

	Ok(token.access_token)
}

fn get_user_file_path() -> Result<PathBuf> {
	Ok(paths::app_data_path()?.join("user.json"))
}

fn is_discord_user_logged_in() -> bool {
	match get_user_file_path() {
		Ok(path) => path.exists(),
		Err(error) => {
			log::error!("Failed to resolve Discord token file path: {error}");
			false
		}
	}
}
