use std::{
	io::{Read, Write},
	net::{TcpListener, TcpStream},
	thread,
	time::Duration,
};

use rai_pal_backend::result::LogErrExt;
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
		match TcpStream::connect(("127.0.0.1", FORWARD_PORT)) {
			Ok(mut stream) => {
				let Some(encoded) = serde_json::to_vec(&arguments)
					.ok_or_log("Failed to serialize arguments for running instance")
				else {
					return;
				};

				match stream.write_all(&encoded) {
					Ok(()) => return,
					Err(err) => {
						log::debug!("Failed to forward arguments to running instance: {err}")
					}
				}
			}
			Err(err) => log::debug!("Failed to connect to running instance: {err}"),
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
			let mut stream = match stream {
				Ok(stream) => stream,
				Err(err) => {
					log::warn!("Failed to accept second instance connection: {err}");
					continue;
				}
			};

			let mut data = String::new();
			if let Err(err) = stream.read_to_string(&mut data) {
				log::warn!("Failed to read second instance arguments: {err}");
				continue;
			}

			let Some(arguments) = serde_json::from_str::<Vec<String>>(&data)
				.ok_or_log("Failed to parse second instance arguments")
			else {
				continue;
			};

			callback(arguments);
		}
	});
}
