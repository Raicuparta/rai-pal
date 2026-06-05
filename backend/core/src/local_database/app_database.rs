use std::sync::{
	Mutex,
	MutexGuard,
};

pub type DbMutex = Mutex<rusqlite::Connection>;

use crate::result::{
	Error,
	Result,
};

pub trait AppDatabase {
	fn lock_db(&self) -> Result<MutexGuard<'_, rusqlite::Connection>>;
}

impl AppDatabase for DbMutex {
	fn lock_db(&self) -> Result<MutexGuard<'_, rusqlite::Connection>> {
		self.lock()
			.map_err(|err| Error::DatabaseLockFailed(err.to_string()))
	}
}
