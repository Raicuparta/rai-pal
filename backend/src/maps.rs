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
	fn try_get<Q: ?Sized>(&self, k: &Q) -> Result<V>
	where
		K: Borrow<Q> + Display,
		Q: Hash + Eq + Display;
}

impl<K, V> TryGet<K, V> for HashMap<K, V>
where
	K: Hash + Eq + Display,
	V: Clone,
{
	fn try_get<Q: ?Sized>(&self, key: &Q) -> Result<V>
	where
		K: Borrow<Q> + Display,
		Q: Hash + Eq + Display,
	{
		self.get(key)
			.ok_or_else(|| Error::DataEntryNotFound(key.to_string()))
			.cloned()
	}
}
