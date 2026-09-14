use std::{
	collections::{HashMap, VecDeque, hash_map::Entry},
	io::ErrorKind,
	sync::{Arc, Mutex, MutexGuard, PoisonError},
	thread,
	time::{Duration, Instant},
};

use opus::{Channels, Decoder, Encoder};

use crate::{proximity_chat::spatial::SpatialOutput, user::auth};

use super::{FRAME_SAMPLES, SAMPLE_RATE};

/// Voice packets use the same host/port as the multiplayer server; UDP and TCP
/// can share a port number.
const VOICE_SERVER_HOST: &str = "everyone.raicuparta.com";
const VOICE_SERVER_PORT: u16 = 1337;

const MAGIC_CLIENT: [u8; 4] = *b"EVVC";
const MAGIC_SERVER: [u8; 4] = *b"EVVS";
const VERSION: u8 = 1;

const KIND_HELLO: u8 = 0;
const KIND_AUDIO: u8 = 1;
const KIND_WELCOME: u8 = 0;

const HEADER_SIZE: usize = 8;
const HELLO_HEADER_SIZE: usize = HEADER_SIZE + 2;
const CLIENT_AUDIO_HEADER_SIZE: usize = HEADER_SIZE + 8 + 4 + 2;
const SERVER_AUDIO_HEADER_SIZE: usize = HEADER_SIZE + 4 + 4 + 3 * 4 + 2;

const MAX_OPUS_SIZE: usize = 1500;
const MAX_PACKET_SIZE: usize = 4096;
const OPUS_MAX_PACKET_SIZE: usize = 4_000;

const HELLO_RETRY_INTERVAL: Duration = Duration::from_secs(2);
const SOCKET_READ_TIMEOUT: Duration = Duration::from_millis(5);
/// Drop a speaker that has been silent for this long.
const SOURCE_TIMEOUT: Duration = Duration::from_secs(2);
/// How much unplayed audio to keep queued per speaker before dropping the oldest.
const MAX_QUEUED_SAMPLES: usize = FRAME_SAMPLES * 25;

/// One remote speaker, with its own Opus decoder (Opus state is per-stream),
/// a small playout queue, and the last position the server reported.
pub struct RemoteSource {
	pub decoder: Decoder,
	pub samples: VecDeque<f32>,
	pub position: [f32; 3],
	pub gain: SpatialOutput,
	pub last_packet: Instant,
}

impl RemoteSource {
	fn new() -> Option<Self> {
		let Ok(decoder) = Decoder::new(SAMPLE_RATE, Channels::Mono) else {
			return None;
		};
		Some(Self {
			decoder,
			samples: VecDeque::new(),
			position: [0.0; 3],
			gain: SpatialOutput::default(),
			last_packet: Instant::now(),
		})
	}
}

pub type RemoteSources = Arc<Mutex<HashMap<i32, RemoteSource>>>;

pub fn remote_sources() -> RemoteSources {
	Arc::new(Mutex::new(HashMap::new()))
}

/// Starts the UDP voice client: authenticates with the server, streams the
/// local mic as Opus, and decodes every remote speaker into `remote_sources`.
pub fn start(mic_buffer: Arc<Mutex<Vec<f32>>>, remote_sources: RemoteSources) {
	thread::spawn(move || {
		if let Err(error) = run(&mic_buffer, &remote_sources) {
			log::error!("Proximity chat voice client failed: {error}");
		}
	});
}

fn run(
	mic_buffer: &Mutex<Vec<f32>>,
	remote_sources: &RemoteSources,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
	let token = auth::read_auth_token().map_err(|error| error.to_string())?;
	if token.trim().is_empty() {
		return Err("no auth token available for voice".into());
	}

	let socket = std::net::UdpSocket::bind("0.0.0.0:0")?;
	socket.connect((VOICE_SERVER_HOST, VOICE_SERVER_PORT))?;
	socket.set_read_timeout(Some(SOCKET_READ_TIMEOUT))?;

	let Ok(mut encoder) = Encoder::new(SAMPLE_RATE, Channels::Mono, opus::Application::Voip) else {
		return Err("failed to create Opus encoder".into());
	};

	let mut opus_buffer = vec![0_u8; OPUS_MAX_PACKET_SIZE];
	let mut mic_frame = Vec::with_capacity(FRAME_SAMPLES);
	let mut session_id: Option<u64> = None;
	let mut last_hello: Option<Instant> = None;
	let mut sequence: u32 = 0;
	let mut buffer = [0_u8; MAX_PACKET_SIZE];

	loop {
		let hello_due = last_hello.is_none_or(|sent| sent.elapsed() >= HELLO_RETRY_INTERVAL);
		if session_id.is_none() && hello_due {
			socket.send(&encode_hello(&token))?;
			last_hello = Some(Instant::now());
		}

		match socket.recv(&mut buffer) {
			Ok(size) => handle_server_packet(&buffer[..size], &mut session_id, remote_sources),
			Err(error) if matches!(error.kind(), ErrorKind::WouldBlock | ErrorKind::TimedOut) => {}
			Err(error) => return Err(error.into()),
		}

		if let Some(id) = session_id
			&& take_mic_frame(mic_buffer, &mut mic_frame)
			&& let Ok(encoded_len) = encoder.encode_float(&mic_frame, &mut opus_buffer)
		{
			socket.send(&encode_audio(id, sequence, &opus_buffer[..encoded_len]))?;
			sequence = sequence.wrapping_add(1);
		}

		expire_sources(remote_sources);
	}
}

fn take_mic_frame(mic_buffer: &Mutex<Vec<f32>>, mic_frame: &mut Vec<f32>) -> bool {
	let mut buffer = lock(mic_buffer);
	if buffer.len() < FRAME_SAMPLES {
		return false;
	}

	mic_frame.clear();
	mic_frame.extend(buffer.drain(0..FRAME_SAMPLES));
	true
}

fn handle_server_packet(data: &[u8], session_id: &mut Option<u64>, remote_sources: &RemoteSources) {
	if data.len() < HEADER_SIZE || data[..4] != MAGIC_SERVER || data[4] != VERSION {
		return;
	}

	match data[5] {
		KIND_WELCOME => {
			if let Some(id) = read_welcome(data) {
				if session_id.is_none() {
					log::info!("Proximity chat voice session {id} established");
				}
				*session_id = Some(id);
			}
		}
		KIND_AUDIO => {
			if let Some((source_id, position, opus)) = read_server_audio(data) {
				decode_into_sources(remote_sources, source_id, position, opus);
			}
		}
		_ => {}
	}
}

#[allow(clippy::significant_drop_tightening)]
fn decode_into_sources(
	remote_sources: &RemoteSources,
	source_id: i32,
	position: [f32; 3],
	opus: &[u8],
) {
	let mut sources = lock(remote_sources);

	if let Entry::Vacant(entry) = sources.entry(source_id) {
		let Some(source) = RemoteSource::new() else {
			log::error!("Failed to create Opus decoder for remote speaker {source_id}");
			return;
		};
		entry.insert(source);
	}

	let Some(source) = sources.get_mut(&source_id) else {
		return;
	};

	source.position = position;
	source.last_packet = Instant::now();

	let mut pcm = vec![0.0_f32; FRAME_SAMPLES];
	let Ok(decoded_len) = source.decoder.decode_float(opus, &mut pcm, false) else {
		return;
	};

	if source.samples.len() > MAX_QUEUED_SAMPLES {
		let excess = source.samples.len() - MAX_QUEUED_SAMPLES;
		source.samples.drain(0..excess);
	}
	source.samples.extend(pcm[..decoded_len].iter().copied());
}

fn expire_sources(remote_sources: &RemoteSources) {
	let now = Instant::now();
	let mut sources = lock(remote_sources);
	sources.retain(|_, source| now.duration_since(source.last_packet) < SOURCE_TIMEOUT);
}

fn encode_hello(token: &str) -> Vec<u8> {
	let Ok(token_len) = u16::try_from(token.len()) else {
		return Vec::new();
	};

	let mut packet = Vec::with_capacity(HELLO_HEADER_SIZE + token.len());
	write_header(&mut packet, KIND_HELLO);
	packet.extend_from_slice(&token_len.to_be_bytes());
	packet.extend_from_slice(token.as_bytes());
	packet
}

fn encode_audio(session_id: u64, sequence: u32, opus: &[u8]) -> Vec<u8> {
	let Ok(opus_len) = u16::try_from(opus.len()) else {
		return Vec::new();
	};

	let mut packet = Vec::with_capacity(CLIENT_AUDIO_HEADER_SIZE + opus.len());
	write_header(&mut packet, KIND_AUDIO);
	packet.extend_from_slice(&session_id.to_be_bytes());
	packet.extend_from_slice(&sequence.to_be_bytes());
	packet.extend_from_slice(&opus_len.to_be_bytes());
	packet.extend_from_slice(opus);
	packet
}

fn write_header(packet: &mut Vec<u8>, kind: u8) {
	packet.extend_from_slice(&MAGIC_CLIENT);
	packet.push(VERSION);
	packet.push(kind);
	packet.extend_from_slice(&[0, 0]);
}

fn read_welcome(data: &[u8]) -> Option<u64> {
	Some(u64::from_be_bytes(read_array::<8>(data, HEADER_SIZE)?))
}

fn read_server_audio(data: &[u8]) -> Option<(i32, [f32; 3], &[u8])> {
	if data.len() < SERVER_AUDIO_HEADER_SIZE {
		return None;
	}

	let source_id = i32::from_be_bytes(read_array::<4>(data, 8)?);
	let position = [
		f32::from_be_bytes(read_array::<4>(data, 16)?),
		f32::from_be_bytes(read_array::<4>(data, 20)?),
		f32::from_be_bytes(read_array::<4>(data, 24)?),
	];
	let opus_len = usize::from(u16::from_be_bytes(read_array::<2>(data, 28)?));
	if opus_len == 0 || opus_len > MAX_OPUS_SIZE {
		return None;
	}

	let end = SERVER_AUDIO_HEADER_SIZE.checked_add(opus_len)?;
	if data.len() < end {
		return None;
	}

	Some((source_id, position, &data[SERVER_AUDIO_HEADER_SIZE..end]))
}

fn read_array<const COUNT: usize>(data: &[u8], offset: usize) -> Option<[u8; COUNT]> {
	let end = offset.checked_add(COUNT)?;
	let Ok(array) = data.get(offset..end)?.try_into() else {
		return None;
	};
	Some(array)
}

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
	mutex.lock().unwrap_or_else(PoisonError::into_inner)
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn hello_roundtrip() {
		let packet = encode_hello("token-123");
		assert_eq!(&packet[..4], &MAGIC_CLIENT);
		assert_eq!(packet[4], VERSION);
		assert_eq!(packet[5], KIND_HELLO);
		let token_len = u16::from_be_bytes([packet[8], packet[9]]) as usize;
		assert_eq!(&packet[10..10 + token_len], b"token-123");
	}

	#[test]
	fn audio_roundtrip() {
		let packet = encode_audio(99, 5, &[9, 8, 7]);
		assert_eq!(packet[5], KIND_AUDIO);
		assert_eq!(
			u64::from_be_bytes([
				packet[8], packet[9], packet[10], packet[11], packet[12], packet[13], packet[14],
				packet[15],
			]),
			99
		);
		assert_eq!(
			u32::from_be_bytes([packet[16], packet[17], packet[18], packet[19]]),
			5
		);
		assert_eq!(u16::from_be_bytes([packet[20], packet[21]]), 3);
		assert_eq!(&packet[22..], &[9, 8, 7]);
	}

	fn server_audio(source_id: i32, position: [f32; 3], opus: &[u8]) -> Vec<u8> {
		let Ok(opus_len) = u16::try_from(opus.len()) else {
			return Vec::new();
		};

		let mut packet = Vec::new();
		packet.extend_from_slice(&MAGIC_SERVER);
		packet.push(VERSION);
		packet.push(KIND_AUDIO);
		packet.extend_from_slice(&[0, 0]);
		packet.extend_from_slice(&source_id.to_be_bytes());
		packet.extend_from_slice(&0_u32.to_be_bytes());
		for value in position {
			packet.extend_from_slice(&value.to_be_bytes());
		}
		packet.extend_from_slice(&opus_len.to_be_bytes());
		packet.extend_from_slice(opus);
		packet
	}

	#[test]
	fn parses_server_audio() {
		let packet = server_audio(7, [1.0, 2.0, 3.0], &[4, 5, 6]);
		let Some((source_id, position, opus)) = read_server_audio(&packet) else {
			panic!("packet should parse");
		};
		assert_eq!(source_id, 7);
		for (actual, expected) in position.iter().zip([1.0_f32, 2.0, 3.0]) {
			assert!((actual - expected).abs() < 1e-6);
		}
		assert_eq!(opus, &[4, 5, 6]);
	}

	#[test]
	fn rejects_truncated_server_audio() {
		let mut packet = server_audio(7, [1.0, 2.0, 3.0], &[4, 5, 6]);
		packet.truncate(SERVER_AUDIO_HEADER_SIZE + 1);
		assert!(read_server_audio(&packet).is_none());
		assert!(read_server_audio(&[0_u8; 4]).is_none());
	}

	#[test]
	fn rejects_empty_server_audio() {
		assert!(read_server_audio(&server_audio(7, [0.0; 3], &[])).is_none());
	}

	#[test]
	fn welcome_roundtrip() {
		let mut packet = Vec::new();
		packet.extend_from_slice(&MAGIC_SERVER);
		packet.push(VERSION);
		packet.push(KIND_WELCOME);
		packet.extend_from_slice(&[0, 0]);
		packet.extend_from_slice(&12345_u64.to_be_bytes());
		assert_eq!(read_welcome(&packet), Some(12345));
		assert_eq!(read_welcome(&packet[..6]), None);
	}
}
