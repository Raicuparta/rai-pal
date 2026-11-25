use std::path::PathBuf;

use crate::{app_paths, database_url, http, path_extensions::PathExt, result::Result};

// The repository over at github.com/Raicuparta/rai-pal-db can have multiple versions of the database.
// This way we prevent old versions of Rai Pal from breaking unless we want them to.
// So when you need to change the database in a backwards-incompatible way,
// you would create a new folder in the database repository and change this number to match the folder.
const DATABASE_VERSION: i32 = 3;

pub fn get_database_file_path() -> Result<PathBuf> {
	app_paths::database_path("remote")
}

pub async fn download_database() -> Result<PathBuf> {
	let url = format!(
		"{}/game-db/{DATABASE_VERSION}/games.db",
		database_url::base_url()
	);

	let file_path = get_database_file_path()?;

	fs_err::tokio::create_dir_all(file_path.try_parent()?).await?;

	http::download_to_file(&url, &file_path).await?;

	Ok(file_path)
}
