use rai_pal_proc_macros::{serde_enum, sql_enum};
use tokio::sync::Mutex as AsyncMutex;

use crate::{
	local_database::{app_database::Db, mod_database::ModDatabase},
	mod_providers::{folder_mod_provider::FolderModProvider, url_mod_provider::UrlModProvider},
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
	async fn refresh(&self, db: &Db) -> Result;
}

#[serde_enum]
#[sql_enum]
pub enum ModProviderId {
	Folder,
	Url,
}

// The whole refresh is guarded so concurrent calls (a toggle racing the
// initial data update, or rapid toggles) can't interleave their insert/remove
// phases and leave stale mods behind. Calls that arrive while a refresh is
// already running are skipped instead of queued, so slow refreshes can't pile
// up.
static REFRESH_LOCK: AsyncMutex<()> = AsyncMutex::const_new(());

/// Refreshes every mod provider, guarded by a module-level lock.
///
/// Does nothing if another refresh is already running. Every provider is
/// attempted even if an earlier one fails, so a broken local mod manifest can't
/// stop the remote (including third party) mod databases from being refreshed.
/// The first error is still returned at the end.
pub async fn refresh_all_mods(db: &Db) -> Result {
	let _guard = match REFRESH_LOCK.try_lock() {
		Ok(guard) => guard,
		Err(err) => {
			log::debug!("Skipping mod refresh because another refresh is running: {err}");
			return Ok(());
		}
	};

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
