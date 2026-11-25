use std::collections::BTreeMap;

use tokio::sync::Mutex as AsyncMutex;

use crate::{
	local_database::{app_database::Db, mod_database::ModDatabase},
	mod_providers::{folder_mod_provider, url_mod_provider},
	result::Result,
};

// The whole refresh is guarded so concurrent calls (a toggle racing the
// initial data update, or rapid toggles) can't interleave their write phases
// and leave stale mods behind. Calls that arrive while a refresh is already
// running are skipped instead of queued, so slow refreshes can't pile up.
static REFRESH_LOCK: AsyncMutex<()> = AsyncMutex::const_new(());

/// Rebuilds the mod catalog from every available source.
///
/// URL sources are merged in priority order, so later sources overwrite mods
/// with the same id from earlier ones. Local folder mods always have the
/// highest priority. Sources that fail to load are skipped for this refresh.
pub async fn refresh_all_mods(db: &Db) -> Result {
	let _guard = match REFRESH_LOCK.try_lock() {
		Ok(guard) => guard,
		Err(err) => {
			log::debug!("Skipping mod refresh because another refresh is running: {err}");
			return Ok(());
		}
	};

	let mut mods = BTreeMap::new();
	let mut tags = BTreeMap::new();

	for url in url_mod_provider::enabled_sources() {
		match url_mod_provider::fetch_mod_database(&url).await {
			Ok(database) => {
				tags.extend(database.tags);
				mods.extend(
					database
						.mods
						.into_iter()
						.map(|game_mod| (game_mod.id.clone(), game_mod)),
				);
			}
			Err(error) => {
				log::warn!("Failed to refresh mod source `{url}`, skipping it: {error}");
			}
		}
	}

	match folder_mod_provider::fetch_mods() {
		Ok(folder_mods) => {
			mods.extend(
				folder_mods
					.into_iter()
					.map(|game_mod| (game_mod.id.clone(), game_mod)),
			);
		}
		Err(error) => log::error!("Failed to load local mods: {error}"),
	}

	db.replace_catalog(&mods.into_values().collect::<Vec<_>>(), &tags)?;
	db.refresh_installed_mods()?;

	Ok(())
}
