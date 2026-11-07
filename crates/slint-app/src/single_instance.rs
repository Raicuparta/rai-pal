use std::{
	io::{Read, Write},
	net::{TcpListener, TcpStream},
	thread,
	time::Duration,
};

use single_instance::SingleInstance;

const INSTANCE_NAME: &str = "rai-pal-single-instance";
const FORWARD_PORT: u16 = 48391;

pub struct Guard {
	_instance: Option<SingleInstance>,
}

pub fn acquire() -> Option<Guard> {
	let instance = match SingleInstance::new(INSTANCE_NAME) {
		Ok(instance) => instance,
		Err(err) => {
			log::warn!("Failed to initialize single instance check: {err}");
			return Some(Guard { _instance: None });
		}
	};

	if instance.is_single() {
		return Some(Guard {
			_instance: Some(instance),
		});
	}

	forward_arguments();
	None
}

fn forward_arguments() {
	let arguments = std::env::args().skip(1).collect::<Vec<_>>();

	for _ in 0..50 {
		if let Ok(mut stream) = TcpStream::connect(("127.0.0.1", FORWARD_PORT)) {
			if stream
				.write_all(&serde_json::to_vec(&arguments).unwrap_or_default())
				.is_ok()
			{
				return;
			}
		}

		thread::sleep(Duration::from_millis(100));
	}

	log::warn!("Failed to forward arguments to the running instance");
}

pub fn listen(callback: impl Fn(Vec<String>) + Send + Sync + 'static) {
	let callback = std::sync::Arc::new(callback);

	thread::spawn(move || {
		let listener = match TcpListener::bind(("127.0.0.1", FORWARD_PORT)) {
			Ok(listener) => listener,
			Err(err) => {
				log::warn!("Failed to listen for second instance arguments: {err}");
				return;
			}
		};

		for stream in listener.incoming() {
			let Ok(mut stream) = stream else {
				continue;
			};

			let mut data = String::new();
			if stream.read_to_string(&mut data).is_err() {
				continue;
			}

			let Ok(arguments) = serde_json::from_str::<Vec<String>>(&data) else {
				continue;
			};

			callback(arguments);
		}
	});
}
