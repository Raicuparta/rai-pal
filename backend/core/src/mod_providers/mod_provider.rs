use rai_pal_proc_macros::serializable_enum;

use crate::{
	local_database::{
		game_database::DbMutex,
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
}

#[serializable_enum]
pub enum ModProviderId {
	Folder,
	Url,
}

async fn refresh_provider<TProvider: ModProvider>(db: &DbMutex) -> Result {
	let start_time = std::time::SystemTime::now()
		.duration_since(std::time::UNIX_EPOCH)?
		.as_secs();

	TProvider::default()?.insert_mods(db).await?;
	db.remove_stale_mods(start_time)?;

	Ok(())
}

pub async fn refresh_all_mods(db: &DbMutex) -> Result {
	refresh_provider::<FolderModProvider>(db).await?;
	refresh_provider::<UrlModProvider>(db).await?;

	db.refresh_installed_mods()?;

	Ok(())
}
