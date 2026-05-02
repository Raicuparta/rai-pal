use std::{
	io::{
		Read,
		Write,
	},
	net::{
		TcpListener,
		TcpStream,
	},
	thread,
	time::Duration,
};

use crate::result::{
		Error,
		Result,
};

use super::discord_oauth;

const USER_SOCKET_BIND_ADDRESS: &str = "127.0.0.1";
const USER_SOCKET_POLL_INTERVAL: Duration = Duration::from_millis(100);

// Important: ports and phrase must be reflected in Everyone Client, or any other mods that rely on this.
const USER_SOCKET_PORT_RANGE_START: u16 = 43950;
const USER_SOCKET_PORT_RANGE_END: u16 = 43960;
const USER_SOCKET_PHRASE: &str = "RAI PAL";

pub fn start_user_socket_manager() {
	thread::spawn(|| {
		let mut listener: Option<TcpListener> = None;
		let mut bind_error_logged = false;

		loop {
			if listener.is_none() {
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

	if path == "/check" {
		write_http_response(stream, 200, "OK", USER_SOCKET_PHRASE)?;
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
			write_http_response(
				stream,
				401,
				"Unauthorized",
				"User is not authenticated in Rai Pal",
			)?;
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
	discord_oauth::read_discord_access_token()
}
