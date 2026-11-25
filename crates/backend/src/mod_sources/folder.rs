use crate::{
	app_paths,
	mods::manifest::{Mod, ModDownload},
	path_extensions::{AsValidStr, PathExt},
	result::{Error, Result},
};

pub fn fetch_mods() -> Result<Vec<Mod>> {
	let folder_path = app_paths::local_mods_path()?;
	let mut mods = Vec::new();

	for manifest_path in folder_path.join("*").join(Mod::FILE_NAME).glob() {
		let Some(mut game_mod) = Mod::from_file(&manifest_path) else {
			continue;
		};

		if game_mod.download.is_some() {
			return Err(Error::LocalModCantDefineDownload(game_mod.id));
		}

		game_mod.download = Some(ModDownload {
			id: "0.0.0".to_string(),
			url: format!("file://{}", manifest_path.try_parent()?.try_to_str()?),
			release_date: None,
		});

		mods.push(game_mod);
	}

	Ok(mods)
}
