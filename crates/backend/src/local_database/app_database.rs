use std::{
	ops::Deref,
	path::PathBuf,
	sync::{
		Mutex, MutexGuard,
		atomic::{AtomicU64, Ordering},
	},
};

use rusqlite::Connection;

use crate::{
	local_database::game_database::open_reader_connection,
	result::{Error, Result},
};

const READER_POOL_MAX: usize = 8;

pub struct Db {
	path: PathBuf,
	writer: Mutex<Connection>,
	readers: Mutex<Vec<Connection>>,
	generation: AtomicU64,
}

impl Db {
	pub fn new(path: PathBuf, writer: Connection) -> Self {
		Self {
			path,
			writer: Mutex::new(writer),
			readers: Mutex::default(),
			generation: AtomicU64::new(0),
		}
	}

	pub fn read_db(&self) -> Result<ReadGuard<'_>> {
		let generation = self.generation.load(Ordering::SeqCst);

		let connection = self
			.readers
			.lock()
			.map_err(|err| Error::DatabaseLockFailed(err.to_string()))?
			.pop();

		let connection = match connection {
			Some(connection) => connection,
			None => open_reader_connection(&self.path)?,
		};

		Ok(ReadGuard {
			db: self,
			generation,
			connection: Some(connection),
		})
	}

	pub fn replace_connection(&self, connection: Connection) -> Result<()> {
		*self
			.writer
			.lock()
			.map_err(|err| Error::DatabaseLockFailed(err.to_string()))? = connection;

		self.generation.fetch_add(1, Ordering::SeqCst);

		self.readers
			.lock()
			.map_err(|err| Error::DatabaseLockFailed(err.to_string()))?
			.clear();

		Ok(())
	}
}

pub trait AppDatabase {
	fn write_db(&self) -> Result<MutexGuard<'_, Connection>>;
}

impl AppDatabase for Db {
	fn write_db(&self) -> Result<MutexGuard<'_, Connection>> {
		self.writer
			.lock()
			.map_err(|err| Error::DatabaseLockFailed(err.to_string()))
	}
}

pub struct ReadGuard<'a> {
	db: &'a Db,
	generation: u64,
	connection: Option<Connection>,
}

impl Deref for ReadGuard<'_> {
	type Target = Connection;

	#[expect(
		clippy::expect_used,
		reason = "the connection is only taken when the guard is dropped"
	)]
	fn deref(&self) -> &Self::Target {
		self.connection
			.as_ref()
			.expect("read guard connection is present until drop")
	}
}

impl Drop for ReadGuard<'_> {
	fn drop(&mut self) {
		let Some(connection) = self.connection.take() else {
			return;
		};

		let Ok(mut readers) = self.db.readers.lock() else {
			return;
		};

		if self.generation == self.db.generation.load(Ordering::SeqCst)
			&& readers.len() < READER_POOL_MAX
		{
			readers.push(connection);
		}
	}
}
