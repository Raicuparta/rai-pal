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
	fn default() -> Self;
	async fn insert_mods(&self, db: &DbMutex) -> Result;
}

pub async fn refresh_all_mods(db: &DbMutex) -> Result {
	let start_time = std::time::SystemTime::now()
		.duration_since(std::time::UNIX_EPOCH)?
		.as_secs();

	FolderModProvider::default().insert_mods(db).await?;
	UrlModProvider::default().insert_mods(db).await?;

	db.refresh_installed_mods()?;
	db.remove_stale_mods(start_time)?;

	Ok(())
}
