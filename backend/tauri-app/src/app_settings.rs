use std::{fs, path::PathBuf};

use crate::result::Result;
use rai_pal_core::paths;
use rai_pal_proc_macros::{serializable_enum, serializable_struct};

#[serializable_enum]
pub enum AppLocale {
	EnUs,
	EsEs,
	FrFr,
	DeDe,
	PtPt,
	ZhCn,
	JaJp,
	KoKr,
}

#[serializable_struct]
#[derive(Default)]
pub struct AppSettings {
	pub hide_game_thumbnails: bool,
	pub override_language: Option<AppLocale>,
}

impl AppSettings {
	fn try_read() -> Result<Self> {
		let path = Self::get_path()?;
		if !path.is_file() {
			return Ok(Self::default());
		}

		let data = fs::read_to_string(&path)?;

		Ok(serde_json::from_str(&data)?)
	}

	pub fn read() -> Self {
		Self::try_read().unwrap_or_else(|err| {
			log::error!("Failed to read app settings, resetting to default. Error: {err}");
			Self::default()
		})
	}

	pub fn try_write(&self) -> Result {
		let path = Self::get_path()?;
		fs::create_dir_all(paths::path_parent(&path)?)?;
		fs::write(&path, serde_json::to_string(self)?)?;

		Ok(())
	}

	fn get_path() -> Result<PathBuf> {
		Ok(paths::app_data_path()?.join("settings.json"))
	}
}
