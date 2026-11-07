use std::{
	fs::{self, File, OpenOptions},
	io::Write,
	path::PathBuf,
	sync::Mutex,
};

use log::{LevelFilter, Metadata, Record};

const MAX_LOG_SIZE: u64 = 10 * 1024 * 1024;

struct Logger {
	file: Mutex<Option<File>>,
	level: LevelFilter,
}

impl log::Log for Logger {
	fn enabled(&self, metadata: &Metadata) -> bool {
		metadata.level() <= self.level
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

		let _ = std::io::stderr().write_all(line.as_bytes());

		if let Ok(mut file) = self.file.lock()
			&& let Some(file) = file.as_mut()
		{
			let _ = file.write_all(line.as_bytes());
		}
	}

	fn flush(&self) {
		if let Ok(mut file) = self.file.lock()
			&& let Some(file) = file.as_mut()
		{
			let _ = file.flush();
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
		OpenOptions::new()
			.create(true)
			.append(true)
			.open(&path)
			.map_err(|err| {
				eprintln!("Failed to open log file {}: {err}", path.display());
			})
			.ok()
	});

	let logger = Logger {
		file: Mutex::new(file),
		level,
	};

	if log::set_boxed_logger(Box::new(logger)).is_ok() {
		log::set_max_level(level);
	}
}

fn log_file() -> Option<PathBuf> {
	let path = rai_pal_core::app_paths::logs_path()
		.map_err(|err| eprintln!("Failed to get logs folder: {err}"))
		.ok()?
		.join("rai-pal.log");
	Some(path)
}

fn rotate_if_needed(path: &PathBuf) {
	let Ok(metadata) = fs::metadata(path) else {
		return;
	};

	if metadata.len() <= MAX_LOG_SIZE {
		return;
	}

	let old_path = path.with_file_name("rai-pal.old.log");
	if let Err(err) = fs::rename(path, old_path) {
		eprintln!("Failed to rotate log file: {err}");
	}
}
