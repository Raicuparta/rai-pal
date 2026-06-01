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

async fn refresh_provider<P: ModProvider>(provider_name: &str, db: &DbMutex) -> bool {
	match P::default().insert_mods(db).await {
		Ok(()) => true,
		Err(err) => {
			log::warn!("Failed to refresh {provider_name} mods: {err}");
			false
		}
	}
}

pub async fn refresh_all_mods(db: &DbMutex) -> Result {
	let start_time = std::time::SystemTime::now()
		.duration_since(std::time::UNIX_EPOCH)?
		.as_secs();

	if refresh_provider::<FolderModProvider>("folder", db).await
		& refresh_provider::<UrlModProvider>("URL", db).await
	{
		db.remove_stale_mods(start_time)?;
	}

	db.refresh_installed_mods()?;

	Ok(())
}
