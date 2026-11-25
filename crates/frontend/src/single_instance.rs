use std::{
	fs::TryLockError,
	io::{self, Read, Write},
	net::{Shutdown, TcpListener, TcpStream},
	sync::Mutex,
	thread,
	time::Duration,
};

use fs_err::{File, OpenOptions};
use rai_pal_backend::{app_paths, result::LogErrExt};

const LOCK_FILE_NAME: &str = "single-instance.lock";
const FORWARD_PORT: u16 = 48391;
const FORWARD_GREETING: &[u8] = b"rai-pal";
const FORWARD_TIMEOUT: Duration = Duration::from_secs(2);
const FORWARD_ATTEMPTS: u8 = 50;
const FORWARD_RETRY_DELAY: Duration = Duration::from_millis(100);

static GUARD: Mutex<Option<File>> = Mutex::new(None);

pub fn acquire() -> bool {
	let Some(lock_file) = open_lock_file() else {
		return true;
	};

	match lock_file.try_lock() {
		Ok(()) => {
			if let Some(mut guard) = GUARD
				.lock()
				.ok_or_log("Failed to lock single instance guard")
			{
				*guard = Some(lock_file);
			}
		}
		Err(TryLockError::WouldBlock) => {
			if forward_arguments() {
				return false;
			}

			log::warn!(
				"Starting anyway: another instance holds the lock but did not accept forwarded arguments"
			);
		}
		Err(TryLockError::Error(err)) => {
			log::warn!("Failed to lock single instance file: {err}");
		}
	}

	true
}

fn open_lock_file() -> Option<File> {
	let path = app_paths::app_data_file(LOCK_FILE_NAME)
		.ok_or_log("Failed to get single instance lock file path")?;

	OpenOptions::new()
		.read(true)
		.write(true)
		.create(true)
		.truncate(false)
		.open(path)
		.ok_or_log("Failed to open single instance lock file")
}

pub fn release() {
	if let Some(mut guard) = GUARD
		.lock()
		.ok_or_log("Failed to lock single instance guard")
	{
		*guard = None;
	}
}

fn forward_arguments() -> bool {
	let arguments = std::env::args().skip(1).collect::<Vec<_>>();
	let Some(encoded) = serde_json::to_vec(&arguments)
		.ok_or_log("Failed to serialize arguments for running instance")
	else {
		return false;
	};

	for _ in 0..FORWARD_ATTEMPTS {
		match try_forward(&encoded) {
			Ok(()) => return true,
			Err(err) => log::debug!("Failed to forward arguments to running instance: {err}"),
		}

		thread::sleep(FORWARD_RETRY_DELAY);
	}

	false
}

fn try_forward(encoded: &[u8]) -> io::Result<()> {
	let mut stream = TcpStream::connect(("127.0.0.1", FORWARD_PORT))?;
	stream.set_read_timeout(Some(FORWARD_TIMEOUT))?;
	stream.set_write_timeout(Some(FORWARD_TIMEOUT))?;

	let mut greeting = [0; FORWARD_GREETING.len()];
	stream.read_exact(&mut greeting)?;
	if greeting != FORWARD_GREETING {
		return Err(io::Error::other(
			"another program is occupying the forwarding port",
		));
	}

	stream.write_all(encoded)?;
	stream.shutdown(Shutdown::Write)
}

pub fn listen(callback: impl Fn(Vec<String>) + Send + Sync + 'static) {
	thread::spawn(move || {
		let listener = match TcpListener::bind(("127.0.0.1", FORWARD_PORT)) {
			Ok(listener) => listener,
			Err(err) => {
				log::warn!("Failed to listen for second instance arguments: {err}");
				return;
			}
		};

		for stream in listener.incoming() {
			let mut stream = match stream {
				Ok(stream) => stream,
				Err(err) => {
					log::warn!("Failed to accept second instance connection: {err}");
					continue;
				}
			};

			if let Err(err) = read_forwarded_arguments(&mut stream, &callback) {
				log::debug!("Failed to read forwarded arguments: {err}");
			}
		}
	});
}

fn read_forwarded_arguments(
	stream: &mut TcpStream,
	callback: &impl Fn(Vec<String>),
) -> io::Result<()> {
	stream.set_read_timeout(Some(FORWARD_TIMEOUT))?;
	stream.set_write_timeout(Some(FORWARD_TIMEOUT))?;
	stream.write_all(FORWARD_GREETING)?;

	let mut data = String::new();
	stream.read_to_string(&mut data)?;

	let Some(arguments) = serde_json::from_str::<Vec<String>>(&data)
		.ok_or_log("Failed to parse second instance arguments")
	else {
		return Ok(());
	};

	callback(arguments);
	Ok(())
}
