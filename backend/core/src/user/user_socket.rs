use std::time::Duration;

use tokio::{
	io::{
		AsyncReadExt,
		AsyncWriteExt,
	},
	net::{
		TcpListener,
		TcpStream,
	},
	time::sleep,
};

use super::auth;
use crate::result::{
	Error,
	Result,
};

const USER_SOCKET_BIND_ADDRESS: &str = "127.0.0.1";
const USER_SOCKET_POLL_INTERVAL: Duration = Duration::from_millis(100);

// Important: ports and phrase must be reflected in Everyone Client, or any other mods that rely on this.
const USER_SOCKET_PORT_RANGE_START: u16 = 43950;
const USER_SOCKET_PORT_RANGE_END: u16 = 43960;
const USER_SOCKET_PHRASE: &str = "RAI PAL";

pub async fn start_user_socket_manager() {
	let mut bind_error_logged = false;

	loop {
		match bind_first_available_port().await {
			Ok((listener, port)) => {
				log::info!("User socket server is listening at {USER_SOCKET_BIND_ADDRESS}:{port}");
				bind_error_logged = false;

				// Continuously await incoming connections without blocking OS threads
				loop {
					match listener.accept().await {
						Ok((mut stream, _)) => {
							if let Err(error) = handle_socket_connection(&mut stream).await {
								log::error!("Failed to handle user socket request: {error}");
							}
						}
						Err(error) => {
							log::error!("User socket accept failed: {error}");
							break; // Break the inner loop to re-bind the listener
						}
					}
				}
			}
			Err(error) => {
				if !bind_error_logged {
					log::error!("Failed to start user socket server: {error}");
					bind_error_logged = true;
				}
			}
		}

		// If binding fails or the accept loop breaks, sleep briefly before retrying
		sleep(USER_SOCKET_POLL_INTERVAL).await;
	}
}

async fn handle_socket_connection(stream: &mut TcpStream) -> Result {
	let mut buffer = [0_u8; 4096];
	let bytes_read = stream.read(&mut buffer).await?;

	if bytes_read == 0 {
		return Ok(());
	}

	let request = String::from_utf8_lossy(&buffer[..bytes_read]);
	let Some(request_line) = request.lines().next() else {
		write_http_response(stream, 400, "Bad Request", "Malformed request").await?;
		return Ok(());
	};

	let mut line_parts = request_line.split_whitespace();
	let method = line_parts.next().unwrap_or_default();
	let path = line_parts.next().unwrap_or_default();

	if method != "GET" {
		write_http_response(stream, 405, "Method Not Allowed", "Only GET is supported").await?;
		return Ok(());
	}

	if path == "/check" {
		write_http_response(stream, 200, "OK", USER_SOCKET_PHRASE).await?;
		return Ok(());
	}

	if path != "/token" {
		write_http_response(stream, 404, "Not Found", "Unknown path").await?;
		return Ok(());
	}

	match read_auth_token() {
		Ok(access_token) => {
			write_http_response(stream, 200, "OK", &access_token).await?;
		}
		Err(error) => {
			write_http_response(
				stream,
				401,
				"Unauthorized",
				"User is not authenticated in Rai Pal",
			)
			.await?;
			log::debug!("Unable to serve /token because token is unavailable: {error}");
		}
	}

	Ok(())
}

async fn write_http_response(
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

	stream.write_all(response.as_bytes()).await?;
	stream.flush().await?;

	Ok(())
}

async fn bind_first_available_port() -> Result<(TcpListener, u16)> {
	for port in USER_SOCKET_PORT_RANGE_START..=USER_SOCKET_PORT_RANGE_END {
		match TcpListener::bind((USER_SOCKET_BIND_ADDRESS, port)).await {
			Ok(listener) => return Ok((listener, port)),
			Err(error) if error.kind() == std::io::ErrorKind::AddrInUse => {}
			Err(error) => {
				return Err(Error::Auth(format!(
					"Failed to bind user socket at {USER_SOCKET_BIND_ADDRESS}:{port}: {error}"
				)));
			}
		}
	}

	Err(Error::Auth(format!(
		"No available user socket ports in range {USER_SOCKET_PORT_RANGE_START}..={USER_SOCKET_PORT_RANGE_END}"
	)))
}

fn read_auth_token() -> Result<String> {
	auth::read_auth_token()
}
