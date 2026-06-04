use rai_pal_proc_macros::serializable_enum;

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
	async fn insert_mods(&self, db: &DbMutex) -> Result;

	fn get_id() -> ModProviderId;

	async fn insert_mods_and_clean(&self, db: &DbMutex) -> Result {
		let start_time = std::time::SystemTime::now()
			.duration_since(std::time::UNIX_EPOCH)?
			.as_secs();

		// Note that if this fails we don't remove stale, and that's by design,
		// since we keep the stale values as a fallback.
		self.insert_mods(db).await?;

		db.remove_stale_mods(start_time, Self::get_id())?;

		Ok(())
	}
}

#[serializable_enum]
pub enum ModProviderId {
	Folder,
	Url,
}

pub async fn refresh_all_mods(db: &DbMutex) -> Result {
	FolderModProvider::default()?
		.insert_mods_and_clean(db)
		.await?;
	UrlModProvider::default()?.insert_mods_and_clean(db).await?;

	db.refresh_installed_mods()?;

	Ok(())
}
