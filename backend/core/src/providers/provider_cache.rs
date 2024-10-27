use std::{collections::HashMap, fs, path::PathBuf};

use crate::result::Result;
use crate::{
	installed_game::InstalledGame,
	owned_game::OwnedGame,
	paths,
	providers::provider::{self, ProviderId},
};
use rai_pal_proc_macros::serializable_struct;

#[serializable_struct]
pub struct ProviderData {
	pub installed_games: HashMap<String, InstalledGame>,
	pub owned_games: HashMap<String, OwnedGame>,
}

pub struct ProviderCache {
	path: PathBuf,
	pub data: ProviderData,
}

impl ProviderCache {
	const CACHE_FOLDER: &'static str = "cache";

	pub fn new(id: ProviderId) -> Result<Self> {
		let folder_path = paths::app_data_path()?
			.join(Self::CACHE_FOLDER)
			.join("providers");

		fs::create_dir_all(&folder_path)?;

		Ok(Self {
			path: folder_path.join(id.to_string()),
			data: ProviderData {
				installed_games: HashMap::new(),
				owned_games: HashMap::new(),
			},
		})
	}

	pub fn load(&mut self) -> Result {
		if !self.path.is_file() {
			return Ok(());
		}

		self.data = serde_json::from_str(&fs::read_to_string(&self.path)?)?;

		Ok(())
	}

	pub fn save(&self) -> Result {
		if self.path.is_dir() {
			fs::remove_dir_all(&self.path)?;
		}

		fs::write(&self.path, serde_json::to_string(&self.data)?)?;

		Ok(())
	}

	pub fn set_data(&mut self, data: ProviderData) -> &mut Self {
		self.data = data;
		self
	}

	pub fn clear(&mut self) -> Result {
		fs::remove_file(&self.path)?;
		self.data = ProviderData {
			installed_games: HashMap::new(),
			owned_games: HashMap::new(),
		};

		Ok(())
	}

	pub fn clear_all() -> Result {
		for provider_id in provider::get_provider_ids() {
			Self::new(provider_id)?.clear()?;
		}

		Ok(())
	}
}
