use std::{
	io::{self, Write},
	path::PathBuf,
	sync::Mutex,
	time::SystemTime,
};

use fs_err::{self as fs, File, OpenOptions};

use log::{LevelFilter, Metadata, Record};
use rai_pal_backend::{
	app_paths,
	result::{Error, Result},
};

struct Logger {
	file: Mutex<Option<File>>,
	level: LevelFilter,
}

impl log::Log for Logger {
	fn enabled(&self, metadata: &Metadata) -> bool {
		if metadata.level() > self.level {
			return false;
		}

		metadata.level() <= LevelFilter::Warn || metadata.target().starts_with("rai_pal")
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

	let file = match session_log_file() {
		Ok(path) => match OpenOptions::new().create(true).append(true).open(&path) {
			Ok(file) => Some(file),
			Err(err) => {
				eprintln!("Failed to open log file {}: {err}", path.display());
				None
			}
		},
		Err(err) => {
			eprintln!("Failed to get logs folder: {err}");
			None
		}
	};

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

fn session_log_file() -> Result<PathBuf> {
	let name = format!(
		"Rai Pal {}.log",
		chrono::Local::now().format("%Y-%m-%d %H-%M-%S")
	);
	Ok(app_paths::logs_path()?.join(name))
}

pub fn latest_log_file() -> Result<PathBuf> {
	let dir = app_paths::logs_path()?;
	let mut latest: Option<(SystemTime, PathBuf)> = None;

	for entry in fs::read_dir(dir)? {
		let entry = entry?;
		let modified = entry.metadata()?.modified()?;

		if latest
			.as_ref()
			.is_none_or(|(latest_modified, _)| modified > *latest_modified)
		{
			latest = Some((modified, entry.path()));
		}
	}

	latest.map(|(_, path)| path).ok_or_else(|| {
		Error::Io(io::Error::new(
			io::ErrorKind::NotFound,
			"No log files found",
		))
	})
}
