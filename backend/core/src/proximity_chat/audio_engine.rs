use std::{
	collections::VecDeque,
	sync::{Arc, Mutex, MutexGuard, PoisonError},
	thread,
	time::Duration,
};

use cpal::{
	Device, SampleFormat, Stream, StreamConfig, SupportedStreamConfigRange,
	traits::{DeviceTrait, HostTrait, StreamTrait},
};

use super::{
	FRAME_SAMPLES, LOOPBACK_PEER_POSITION, SAMPLE_RATE, SharedTransform,
	spatial::{self, SpatialConfig},
	voice::{self, RemoteSources},
};

/// Opus frames are at most 1275 bytes, but leave some headroom.
const OPUS_MAX_PACKET_SIZE: usize = 4_000;
/// How much unplayed audio to keep queued before dropping the oldest samples.
const MAX_QUEUED_SAMPLES: usize = FRAME_SAMPLES * 25;

type AudioResult<T = ()> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

/// Where the output mixer reads its audio from.
#[derive(Clone)]
enum Mixer {
	/// Self-test: the local mic encoded and decoded straight back as one peer.
	Loopback(Arc<Mutex<VecDeque<f32>>>),
	/// Real server: one spatialized stream per remote speaker.
	Network(RemoteSources),
}

/// Starts the experimental proximity chat audio engine.
///
/// By default it streams the local mic to the Everyone voice server and plays
/// back each remote speaker with client-side 3D spatialization. Set
/// `RAI_PAL_PROXIMITY_LOOPBACK=1` to use the local loopback self-test instead.
pub fn start(transform: Arc<SharedTransform>) {
	thread::spawn(move || {
		let loopback = std::env::var("RAI_PAL_PROXIMITY_LOOPBACK").is_ok();
		if let Err(error) = run(transform, loopback) {
			log::error!("Proximity chat audio engine failed: {error}");
		}
	});
}

fn run(transform: Arc<SharedTransform>, loopback: bool) -> AudioResult {
	let host = cpal::default_host();
	let input_device = host
		.default_input_device()
		.ok_or("no default input device available")?;
	let output_device = host
		.default_output_device()
		.ok_or("no default output device available")?;

	let input_config = pick_config(input_device.supported_input_configs()?, 1)
		.ok_or("no 48 kHz float input config available")?;
	let output_config = pick_config(output_device.supported_output_configs()?, 2)
		.ok_or("no 48 kHz float output config available")?;

	log::info!(
		"Proximity chat input: {} channel(s) @ {} Hz",
		input_config.channels,
		input_config.sample_rate
	);
	log::info!(
		"Proximity chat output: {} channel(s) @ {} Hz",
		output_config.channels,
		output_config.sample_rate
	);

	let mic_buffer = Arc::new(Mutex::new(Vec::<f32>::new()));

	let mixer = if loopback {
		let queue = Arc::new(Mutex::new(VecDeque::<f32>::new()));
		spawn_loopback_processor(Arc::clone(&mic_buffer), Arc::clone(&queue));
		log::info!("Proximity chat audio engine started (loopback self-test)");
		Mixer::Loopback(queue)
	} else {
		let remote_sources = voice::remote_sources();
		voice::start(Arc::clone(&mic_buffer), Arc::clone(&remote_sources));
		log::info!("Proximity chat audio engine started (server voice)");
		Mixer::Network(remote_sources)
	};

	let input_stream = build_input_stream(&input_device, input_config, mic_buffer)?;
	let output_stream = build_output_stream(&output_device, output_config, mixer, transform)?;

	input_stream.play()?;
	output_stream.play()?;

	// The cpal streams own the audio threads, but dropping the `Stream` handles
	// stops them, and `Stream` isn't `Send` so it can't be stored elsewhere.
	// Leak the handles to keep playback alive for the lifetime of the process.
	std::mem::forget(input_stream);
	std::mem::forget(output_stream);

	Ok(())
}

/// Picks the first 48 kHz float config, preferring `preferred_channels` so the
/// output stream stays stereo (mono would collapse stereo panning).
fn pick_config(
	configs: impl Iterator<Item = SupportedStreamConfigRange>,
	preferred_channels: u16,
) -> Option<StreamConfig> {
	let mut fallback = None;

	for config in configs {
		if config.sample_format() != SampleFormat::F32 {
			continue;
		}

		let Some(supported) = config.try_with_sample_rate(SAMPLE_RATE) else {
			continue;
		};

		let stream_config = StreamConfig::from(supported);
		if stream_config.channels == preferred_channels {
			return Some(stream_config);
		}

		if fallback.is_none() {
			fallback = Some(stream_config);
		}
	}

	fallback
}

fn build_input_stream(
	device: &Device,
	config: StreamConfig,
	mic_buffer: Arc<Mutex<Vec<f32>>>,
) -> AudioResult<Stream> {
	let channels = usize::from(config.channels).max(1);
	let divisor = f32::from(config.channels.max(1));

	let stream = device.build_input_stream(
		config,
		move |data: &[f32], _| {
			let mut buffer = lock(&mic_buffer);

			for frame in data.chunks(channels) {
				let sum: f32 = frame.iter().sum();
				buffer.push(sum / divisor);
			}

			// If the encoder thread ever stalls, drop the oldest audio instead
			// of growing forever.
			let max_buffered = FRAME_SAMPLES * 100;
			if buffer.len() > max_buffered {
				let excess = buffer.len() - max_buffered;
				buffer.drain(0..excess);
			}
		},
		|error| log::error!("Proximity chat input stream error: {error}"),
		None,
	)?;

	Ok(stream)
}

fn build_output_stream(
	device: &Device,
	config: StreamConfig,
	mixer: Mixer,
	transform: Arc<SharedTransform>,
) -> AudioResult<Stream> {
	let channels = usize::from(config.channels).max(1);
	let spatial_config = SpatialConfig::default();

	let stream = device.build_output_stream(
		config,
		move |data: &mut [f32], _| {
			// Recompute gains once per callback; players can't move far in the
			// span of a single audio buffer.
			let listener = transform.get().unwrap_or_default();

			match &mixer {
				Mixer::Loopback(queue) => {
					let source = transform.get_source().unwrap_or(LOOPBACK_PEER_POSITION);
					let spatial = spatial::spatialize(listener, source, &spatial_config);
					mix_loopback(data, channels, queue, spatial);
				}
				Mixer::Network(remote_sources) => {
					mix_network(data, channels, remote_sources, listener, spatial_config);
				}
			}
		},
		|error| log::error!("Proximity chat output stream error: {error}"),
		None,
	)?;

	Ok(stream)
}

#[allow(clippy::significant_drop_tightening)]
fn mix_loopback(
	data: &mut [f32],
	channels: usize,
	queue: &Mutex<VecDeque<f32>>,
	spatial: spatial::SpatialOutput,
) {
	let mut queue = lock(queue);

	for frame in data.chunks_mut(channels) {
		let sample = queue.pop_front().unwrap_or(0.0);
		write_stereo(
			frame,
			channels,
			sample * spatial.left,
			sample * spatial.right,
		);
	}
}

#[allow(clippy::significant_drop_tightening)]
fn mix_network(
	data: &mut [f32],
	channels: usize,
	remote_sources: &RemoteSources,
	listener: crate::proximity_chat::Transform,
	config: SpatialConfig,
) {
	let mut sources = lock(remote_sources);

	for source in sources.values_mut() {
		source.gain = spatial::spatialize(listener, source.position, &config);
	}

	for frame in data.chunks_mut(channels) {
		let mut left = 0.0;
		let mut right = 0.0;

		for source in sources.values_mut() {
			let Some(sample) = source.samples.pop_front() else {
				continue;
			};
			left = sample.mul_add(source.gain.left, left);
			right = sample.mul_add(source.gain.right, right);
		}

		write_stereo(frame, channels, left, right);
	}
}

fn write_stereo(frame: &mut [f32], channels: usize, left: f32, right: f32) {
	if channels == 1 {
		frame[0] = ((left + right) * 0.5).clamp(-1.0, 1.0);
	} else {
		frame[0] = left.clamp(-1.0, 1.0);
		frame[1] = right.clamp(-1.0, 1.0);
		for silent in frame.iter_mut().skip(2) {
			*silent = 0.0;
		}
	}
}

fn spawn_loopback_processor(
	mic_buffer: Arc<Mutex<Vec<f32>>>,
	playback_queue: Arc<Mutex<VecDeque<f32>>>,
) {
	thread::spawn(move || {
		let (Ok(mut encoder), Ok(mut decoder)) = (
			opus::Encoder::new(SAMPLE_RATE, opus::Channels::Mono, opus::Application::Voip),
			opus::Decoder::new(SAMPLE_RATE, opus::Channels::Mono),
		) else {
			log::error!("Failed to create Opus encoder/decoder for proximity chat");
			return;
		};

		let mut mic_frame = Vec::with_capacity(FRAME_SAMPLES);
		let mut opus_packet = vec![0_u8; OPUS_MAX_PACKET_SIZE];
		let mut pcm_frame = vec![0.0_f32; FRAME_SAMPLES];

		loop {
			let mut has_frame = false;
			let mut buffer = lock(&mic_buffer);
			if buffer.len() >= FRAME_SAMPLES {
				mic_frame.clear();
				mic_frame.extend(buffer.drain(0..FRAME_SAMPLES));
				has_frame = true;
			}
			drop(buffer);

			if !has_frame {
				thread::sleep(Duration::from_millis(5));
				continue;
			}

			let Ok(packet_len) = encoder.encode_float(&mic_frame, &mut opus_packet) else {
				log::error!("Failed to encode microphone audio for proximity chat");
				continue;
			};

			let Ok(pcm_len) =
				decoder.decode_float(&opus_packet[..packet_len], &mut pcm_frame, false)
			else {
				log::error!("Failed to decode proximity chat audio");
				continue;
			};

			let mut queue = lock(&playback_queue);
			if queue.len() > MAX_QUEUED_SAMPLES {
				let excess = queue.len() - MAX_QUEUED_SAMPLES;
				queue.drain(0..excess);
			}
			queue.extend(pcm_frame[..pcm_len].iter().copied());
		}
	});
}

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
	mutex.lock().unwrap_or_else(PoisonError::into_inner)
}
