use std::{
	borrow::Borrow,
	collections::HashMap,
	fmt::Display,
	hash::{
		BuildHasher,
		Hash,
	},
};

use anyhow::Context;

use crate::result::{
	CoreError,
	CoreResult,
};

pub trait TryGettable<K, V> {
	fn try_get<Q>(&self, k: &Q) -> CoreResult<&V>
	where
		K: Borrow<Q> + Display,
		Q: Hash + Eq + Display + ?Sized;

	fn try_get_mut<Q>(&mut self, k: &Q) -> CoreResult<&mut V>
	where
		K: Borrow<Q> + Display,
		Q: Hash + Eq + Display + ?Sized;
}

impl<K, V, S: BuildHasher> TryGettable<K, V> for HashMap<K, V, S>
where
	K: Hash + Eq + Display,
{
	fn try_get<Q>(&self, key: &Q) -> CoreResult<&V>
	where
		K: Borrow<Q> + Display,
		Q: Hash + Eq + Display + ?Sized,
	{
		self.get(key)
			.with_context(|| CoreError::DataEntryNotFound(key.to_string()))
	}

	fn try_get_mut<Q>(&mut self, key: &Q) -> CoreResult<&mut V>
	where
		K: Borrow<Q> + Display,
		Q: Hash + Eq + Display + ?Sized,
	{
		self.get_mut(key)
			.with_context(|| CoreError::DataEntryNotFound(key.to_string()))
	}
}
