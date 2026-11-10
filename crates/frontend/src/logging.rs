use std::{
	io::{self, Write},
	path::PathBuf,
	sync::Mutex,
};

use fs_err::{self as fs, File, OpenOptions};

use log::{LevelFilter, Metadata, Record};

const MAX_LOG_SIZE: u64 = 10 * 1024 * 1024;

struct Logger {
	file: Mutex<Option<File>>,
	level: LevelFilter,
}

impl log::Log for Logger {
	fn enabled(&self, metadata: &Metadata) -> bool {
		if metadata.level() > self.level {
			return false;
		}

		metadata.level() <= LevelFilter::Info || metadata.target().starts_with("rai_pal")
	}

	fn log(&self, record: &Record) {
		if !self.enabled(record.metadata()) {
			return;
		}

		let line = format!(
			"{} [{}] {}: {}\n",
			chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f"),
			record.level(),
			record.target(),
			record.args()
		);

		if let Err(err) = std::io::stderr().write_all(line.as_bytes()) {
			eprintln!("Failed to write log line to stderr: {err}");
		}

		match self.file.lock() {
			Ok(mut file) => {
				if let Some(file) = file.as_mut()
					&& let Err(err) = file.write_all(line.as_bytes())
				{
					eprintln!("Failed to write log line to file: {err}");
				}
			}
			Err(err) => eprintln!("Failed to lock log file: {err}"),
		}
	}

	fn flush(&self) {
		match self.file.lock() {
			Ok(mut file) => {
				if let Some(file) = file.as_mut()
					&& let Err(err) = file.flush()
				{
					eprintln!("Failed to flush log file: {err}");
				}
			}
			Err(err) => eprintln!("Failed to lock log file: {err}"),
		}
	}
}

pub fn init() {
	let level = if cfg!(debug_assertions) {
		LevelFilter::Trace
	} else {
		LevelFilter::Info
	};

	let file = log_file().and_then(|path| {
		rotate_if_needed(&path);
		match OpenOptions::new().create(true).append(true).open(&path) {
			Ok(file) => Some(file),
			Err(err) => {
				eprintln!("Failed to open log file {}: {err}", path.display());
				None
			}
		}
	});

	let logger = Logger {
		file: Mutex::new(file),
		level,
	};

	if let Err(err) = log::set_boxed_logger(Box::new(logger)) {
		eprintln!("Failed to install logger: {err}");
	} else {
		log::set_max_level(level);
	}
}

fn log_file() -> Option<PathBuf> {
	let path = match rai_pal_backend::app_paths::logs_path() {
		Ok(path) => path,
		Err(err) => {
			eprintln!("Failed to get logs folder: {err}");
			return None;
		}
	}
	.join("rai-pal.log");
	Some(path)
}

fn rotate_if_needed(path: &PathBuf) {
	let metadata = match fs::metadata(path) {
		Ok(metadata) => metadata,
		Err(err) if err.kind() == io::ErrorKind::NotFound => return,
		Err(err) => {
			eprintln!("Failed to read log file metadata {}: {err}", path.display());
			return;
		}
	};

	if metadata.len() <= MAX_LOG_SIZE {
		return;
	}

	let old_path = path.with_file_name("rai-pal.old.log");
	if let Err(err) = fs::rename(path, old_path) {
		eprintln!("Failed to rotate log file: {err}");
	}
}
