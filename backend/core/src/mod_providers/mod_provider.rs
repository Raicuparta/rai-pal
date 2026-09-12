use rai_pal_proc_macros::serializable_enum;
use tokio::sync::Mutex as AsyncMutex;

use crate::{
	local_database::{
		app_database::DbMutex,
		mod_database::ModDatabase,
	},
	mod_providers::{
		folder_mod_provider::FolderModProvider,
		url_mod_provider::UrlModProvider,
	},
	result::Result,
};

pub trait ModProvider {
	fn default() -> Result<Self>
	where
		Self: Sized;
	fn get_id() -> ModProviderId;

	/// Reconciles this provider's mods in the database.
	///
	/// Fetches mods from enabled sources (or manifests present on disk) and
	/// removes anything that should no longer exist. Unlike the previous
	/// timestamp-based approach, removal is deterministic and based on source
	/// identity, so it doesn't race with other refreshes or depend on wall-clock
	/// timing.
	async fn refresh(&self, db: &DbMutex) -> Result;
}

#[serializable_enum]
pub enum ModProviderId {
	Folder,
	Url,
}

// The whole refresh is serialized so concurrent calls (a toggle racing the
// initial data update, or rapid toggles) can't interleave their insert/remove
// phases and leave stale mods behind.
static REFRESH_LOCK: AsyncMutex<()> = AsyncMutex::const_new(());

/// Refreshes every mod provider, serialized by a module-level lock.
///
/// Every provider is attempted even if an earlier one fails, so a broken local
/// mod manifest can't stop the remote (including third party) mod databases
/// from being refreshed. The first error is still returned at the end.
pub async fn refresh_all_mods(db: &DbMutex) -> Result {
	let _guard = REFRESH_LOCK.lock().await;

	let mut first_error = if let Err(error) = FolderModProvider::default()?.refresh(db).await {
		log::error!("Failed to refresh local mods: {error}");
		Some(error)
	} else {
		None
	};

	if let Err(error) = UrlModProvider::default()?.refresh(db).await {
		log::error!("Failed to refresh URL mod sources: {error}");
		first_error.get_or_insert(error);
	}

	db.refresh_installed_mods()?;

	first_error.map_or(Ok(()), Err)
}
