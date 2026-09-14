use std::{sync::Arc, time::Duration};

use serde::Deserialize;
use tokio::{net::UdpSocket, time::sleep};

use super::{POSITION_SOCKET_BIND_ADDRESS, POSITION_SOCKET_PORT, SharedTransform, Transform};

/// `RPAL` magic, then a version byte and 3 reserved bytes.
const PACKET_MAGIC: [u8; 4] = *b"RPAL";
/// Listener only: position + rotation.
const PACKET_VERSION: u8 = 1;
/// Listener + a separate source position (temporary loopback testing).
const PACKET_VERSION_WITH_SOURCE: u8 = 2;
const PACKET_HEADER_SIZE: usize = 8;
const PACKET_SIZE: usize = PACKET_HEADER_SIZE + 6 * size_of::<f32>();
const PACKET_SIZE_WITH_SOURCE: usize = PACKET_SIZE + 3 * size_of::<f32>();

const MAX_PACKET_SIZE: usize = 1024;
const BIND_RETRY_INTERVAL: Duration = Duration::from_millis(500);

#[derive(Clone, Copy, Debug, PartialEq)]
struct PositionUpdate {
	listener: Transform,
	/// When present, the loopback audio should be spatialized as coming from
	/// this position instead of the fallback simulated peer.
	source: Option<[f32; 3]>,
}

#[derive(Deserialize)]
struct JsonPacket {
	position: [f32; 3],
	#[serde(default)]
	rotation: [f32; 3],
	#[serde(default)]
	source: Option<[f32; 3]>,
}

pub async fn run(transform: Arc<SharedTransform>) {
	let mut bind_error_logged = false;

	loop {
		match UdpSocket::bind((POSITION_SOCKET_BIND_ADDRESS, POSITION_SOCKET_PORT)).await {
			Ok(socket) => {
				log::info!(
					"Proximity chat position socket listening at {POSITION_SOCKET_BIND_ADDRESS}:{POSITION_SOCKET_PORT}"
				);
				bind_error_logged = false;
				receive_loop(socket, &transform).await;
			}
			Err(error) => {
				if !bind_error_logged {
					log::error!("Failed to bind proximity chat position socket: {error}");
					bind_error_logged = true;
				}
			}
		}

		sleep(BIND_RETRY_INTERVAL).await;
	}
}

async fn receive_loop(socket: UdpSocket, transform: &SharedTransform) {
	let mut buffer = [0_u8; MAX_PACKET_SIZE];

	loop {
		match socket.recv_from(&mut buffer).await {
			Ok((bytes_read, _source)) => {
				if let Some(update) = parse_packet(&buffer[..bytes_read]) {
					transform.set(update.listener);
					match update.source {
						Some(source) => transform.set_source(source),
						None => transform.clear_source(),
					}
				}
			}
			Err(error) => {
				log::error!("Proximity chat position socket receive failed: {error}");
				return;
			}
		}
	}
}

fn parse_packet(data: &[u8]) -> Option<PositionUpdate> {
	if data.starts_with(&PACKET_MAGIC) {
		return parse_binary_packet(data);
	}

	// Convenience fallback so a peer can be simulated with a plain UDP text
	// message, e.g. `{"position":[0,0,0],"rotation":[0,1.57,0]}`.
	parse_json_packet(data)
}

fn parse_binary_packet(data: &[u8]) -> Option<PositionUpdate> {
	let has_source = match *data.get(4)? {
		PACKET_VERSION => false,
		PACKET_VERSION_WITH_SOURCE => true,
		_ => return None,
	};

	let expected_size = if has_source {
		PACKET_SIZE_WITH_SOURCE
	} else {
		PACKET_SIZE
	};
	if data.len() < expected_size {
		return None;
	}

	let listener_floats = read_floats::<6>(data, PACKET_HEADER_SIZE)?;
	let listener = Transform {
		position: [listener_floats[0], listener_floats[1], listener_floats[2]],
		rotation: [listener_floats[3], listener_floats[4], listener_floats[5]],
	};

	let source = if has_source {
		let source_floats = read_floats::<3>(data, PACKET_HEADER_SIZE + 6 * size_of::<f32>())?;
		Some([source_floats[0], source_floats[1], source_floats[2]])
	} else {
		None
	};

	Some(PositionUpdate { listener, source })
}

fn parse_json_packet(data: &[u8]) -> Option<PositionUpdate> {
	let Ok(packet) = serde_json::from_slice::<JsonPacket>(data) else {
		return None;
	};
	if !is_finite(&packet.position) || !is_finite(&packet.rotation) {
		return None;
	}
	if let Some(source) = &packet.source
		&& !is_finite(source)
	{
		return None;
	}

	Some(PositionUpdate {
		listener: Transform {
			position: packet.position,
			rotation: packet.rotation,
		},
		source: packet.source,
	})
}

fn read_floats<const COUNT: usize>(data: &[u8], offset: usize) -> Option<[f32; COUNT]> {
	let mut floats = [0.0_f32; COUNT];
	for (index, value) in floats.iter_mut().enumerate() {
		let start = offset + index * size_of::<f32>();
		let Ok(bytes) = data.get(start..start + size_of::<f32>())?.try_into() else {
			return None;
		};
		*value = f32::from_le_bytes(bytes);
	}
	Some(floats)
}

fn is_finite(values: &[f32; 3]) -> bool {
	values.iter().all(|value| value.is_finite())
}

#[cfg(test)]
mod tests {
	use super::*;

	fn binary_packet(
		version: u8,
		position: [f32; 3],
		rotation: [f32; 3],
		source: Option<[f32; 3]>,
	) -> Vec<u8> {
		let mut packet = Vec::new();
		packet.extend_from_slice(&PACKET_MAGIC);
		packet.push(version);
		packet.extend_from_slice(&[0, 0, 0]);
		for value in position.into_iter().chain(rotation) {
			packet.extend_from_slice(&value.to_le_bytes());
		}
		if let Some(source) = source {
			for value in source {
				packet.extend_from_slice(&value.to_le_bytes());
			}
		}
		packet
	}

	fn listener(position: [f32; 3], rotation: [f32; 3]) -> Transform {
		Transform { position, rotation }
	}

	#[test]
	fn parses_binary_packet_without_source() {
		let packet = binary_packet(PACKET_VERSION, [1.0, 2.0, 3.0], [0.1, 0.2, 0.3], None);
		assert_eq!(
			parse_packet(&packet),
			Some(PositionUpdate {
				listener: listener([1.0, 2.0, 3.0], [0.1, 0.2, 0.3]),
				source: None,
			})
		);
	}

	#[test]
	fn parses_binary_packet_with_source() {
		let packet = binary_packet(
			PACKET_VERSION_WITH_SOURCE,
			[1.0, 2.0, 3.0],
			[0.1, 0.2, 0.3],
			Some([7.0, 8.0, 9.0]),
		);
		assert_eq!(
			parse_packet(&packet),
			Some(PositionUpdate {
				listener: listener([1.0, 2.0, 3.0], [0.1, 0.2, 0.3]),
				source: Some([7.0, 8.0, 9.0]),
			})
		);
	}

	#[test]
	fn rejects_truncated_binary_packet() {
		let mut packet = binary_packet(PACKET_VERSION, [1.0, 2.0, 3.0], [0.0, 0.0, 0.0], None);
		packet.truncate(PACKET_SIZE - 1);
		assert_eq!(parse_packet(&packet), None);
	}

	#[test]
	fn rejects_truncated_source_packet() {
		let mut packet = binary_packet(
			PACKET_VERSION_WITH_SOURCE,
			[1.0, 2.0, 3.0],
			[0.0, 0.0, 0.0],
			Some([7.0, 8.0, 9.0]),
		);
		packet.truncate(PACKET_SIZE_WITH_SOURCE - 1);
		assert_eq!(parse_packet(&packet), None);
	}

	#[test]
	fn rejects_wrong_version() {
		let mut packet = binary_packet(PACKET_VERSION, [1.0, 2.0, 3.0], [0.0, 0.0, 0.0], None);
		packet[4] = 99;
		assert_eq!(parse_packet(&packet), None);
	}

	#[test]
	fn parses_json_packet() {
		let packet = br#"{"position":[1,2,3],"rotation":[0,1.5,0]}"#;
		assert_eq!(
			parse_packet(packet),
			Some(PositionUpdate {
				listener: listener([1.0, 2.0, 3.0], [0.0, 1.5, 0.0]),
				source: None,
			})
		);
	}

	#[test]
	fn parses_json_packet_with_source() {
		let packet = br#"{"position":[1,2,3],"rotation":[0,1.5,0],"source":[7,8,9]}"#;
		assert_eq!(
			parse_packet(packet),
			Some(PositionUpdate {
				listener: listener([1.0, 2.0, 3.0], [0.0, 1.5, 0.0]),
				source: Some([7.0, 8.0, 9.0]),
			})
		);
	}

	#[test]
	fn json_rotation_defaults_to_zero() {
		let packet = br#"{"position":[1,2,3]}"#;
		assert_eq!(
			parse_packet(packet),
			Some(PositionUpdate {
				listener: listener([1.0, 2.0, 3.0], [0.0, 0.0, 0.0]),
				source: None,
			})
		);
	}

	#[test]
	fn ignores_garbage() {
		assert_eq!(parse_packet(b"not a packet"), None);
	}

	#[test]
	fn shared_transform_roundtrip() {
		let shared = SharedTransform::new();
		assert_eq!(shared.get(), None);
		assert_eq!(shared.get_source(), None);

		let transform = Transform {
			position: [4.0, 5.0, 6.0],
			rotation: [0.5, 1.0, 1.5],
		};
		shared.set(transform);
		assert_eq!(shared.get(), Some(transform));

		shared.set_source([7.0, 8.0, 9.0]);
		assert_eq!(shared.get_source(), Some([7.0, 8.0, 9.0]));

		shared.clear_source();
		assert_eq!(shared.get_source(), None);
	}
}
