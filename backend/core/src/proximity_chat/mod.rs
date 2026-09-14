pub mod audio_engine;
pub mod position_socket;
pub mod spatial;
pub mod voice;

use std::sync::{
	Arc, OnceLock,
	atomic::{AtomicBool, AtomicU32, Ordering},
};

/// Experimental, temporary feature: hardcoded for now.
pub const POSITION_SOCKET_BIND_ADDRESS: &str = "127.0.0.1";
pub const POSITION_SOCKET_PORT: u16 = 6767;

/// Fixed location of the simulated peer used by the loopback self-test when
/// no source position is being streamed.
pub const LOOPBACK_PEER_POSITION: [f32; 3] = [0.0, 0.0, -4.0];

/// Opus and cpal operate at 48 kHz mono, in 20 ms frames.
pub const SAMPLE_RATE: u32 = 48_000;
pub const FRAME_SAMPLES: usize = 960;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Transform {
	pub position: [f32; 3],
	/// Euler angles in radians: pitch (x), yaw (y), roll (z).
	pub rotation: [f32; 3],
}

/// Latest listener transform, exposed atomically so the real-time audio
/// callback never has to take a lock that the network task could hold.
pub struct SharedTransform {
	position: [AtomicU32; 3],
	rotation: [AtomicU32; 3],
	has_data: AtomicBool,
	source_position: [AtomicU32; 3],
	has_source: AtomicBool,
}

impl SharedTransform {
	const fn new() -> Self {
		Self {
			position: [const { AtomicU32::new(0) }; 3],
			rotation: [const { AtomicU32::new(0) }; 3],
			has_data: AtomicBool::new(false),
			source_position: [const { AtomicU32::new(0) }; 3],
			has_source: AtomicBool::new(false),
		}
	}

	pub fn set(&self, transform: Transform) {
		for (slot, value) in self.position.iter().zip(transform.position) {
			slot.store(value.to_bits(), Ordering::Relaxed);
		}
		for (slot, value) in self.rotation.iter().zip(transform.rotation) {
			slot.store(value.to_bits(), Ordering::Relaxed);
		}
		self.has_data.store(true, Ordering::Release);
	}

	pub fn get(&self) -> Option<Transform> {
		if !self.has_data.load(Ordering::Acquire) {
			return None;
		}

		let mut position = [0.0; 3];
		for (slot, value) in self.position.iter().zip(position.iter_mut()) {
			*value = f32::from_bits(slot.load(Ordering::Relaxed));
		}

		let mut rotation = [0.0; 3];
		for (slot, value) in self.rotation.iter().zip(rotation.iter_mut()) {
			*value = f32::from_bits(slot.load(Ordering::Relaxed));
		}

		Some(Transform { position, rotation })
	}

	/// Temporary testing hook: the position the loopback audio should appear to
	/// come from (e.g. another player), while `get` stays the listener.
	pub fn set_source(&self, source_position: [f32; 3]) {
		for (slot, value) in self.source_position.iter().zip(source_position) {
			slot.store(value.to_bits(), Ordering::Relaxed);
		}
		self.has_source.store(true, Ordering::Release);
	}

	pub fn clear_source(&self) {
		self.has_source.store(false, Ordering::Release);
	}

	pub fn get_source(&self) -> Option<[f32; 3]> {
		if !self.has_source.load(Ordering::Acquire) {
			return None;
		}

		let mut source_position = [0.0; 3];
		for (slot, value) in self.source_position.iter().zip(source_position.iter_mut()) {
			*value = f32::from_bits(slot.load(Ordering::Relaxed));
		}

		Some(source_position)
	}
}

static SHARED_TRANSFORM: OnceLock<Arc<SharedTransform>> = OnceLock::new();

pub fn shared_transform() -> Arc<SharedTransform> {
	Arc::clone(SHARED_TRANSFORM.get_or_init(|| Arc::new(SharedTransform::new())))
}

pub fn start_proximity_chat() {
	let transform = shared_transform();
	tokio::spawn(position_socket::run(Arc::clone(&transform)));
	audio_engine::start(transform);
}
