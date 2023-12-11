use std::{
	borrow::Borrow,
	collections::HashMap,
	fmt::Display,
	hash::Hash,
};

use crate::{
	Error,
	Result,
};

pub trait TryGet<K, V> {
	// TODO return reference, separate impl for Mutex.
	fn try_get<Q>(&self, k: &Q) -> Result<V>
	where
		K: Borrow<Q> + Display,
		Q: Hash + Eq + Display + ?Sized;
}

impl<K, V> TryGet<K, V> for HashMap<K, V>
where
	K: Hash + Eq + Display,
	V: Clone,
{
	fn try_get<Q>(&self, key: &Q) -> Result<V>
	where
		K: Borrow<Q> + Display,
		Q: Hash + Eq + Display + ?Sized,
	{
		self.get(key)
			.ok_or_else(|| Error::DataEntryNotFound(key.to_string()))
			.cloned()
	}
}

pub trait TryGetMut<K, V> {
	fn try_get_mut<Q>(&mut self, k: &Q) -> Result<&mut V>
	where
		K: Borrow<Q> + Display,
		Q: Hash + Eq + Display + ?Sized;
}

impl<K, V> TryGetMut<K, V> for HashMap<K, V>
where
	K: Hash + Eq + Display,
	V: Clone,
{
	fn try_get_mut<Q>(&mut self, key: &Q) -> Result<&mut V>
	where
		K: Borrow<Q> + Display,
		Q: Hash + Eq + Display + ?Sized,
	{
		self.get_mut(key)
			.ok_or_else(|| Error::DataEntryNotFound(key.to_string()))
	}
}
