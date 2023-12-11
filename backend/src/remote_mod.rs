use std::collections::HashMap;

use crate::{
	game_mod::CommonModData,
	mod_loaders::mod_database::ModDownload,
	serializable_struct,
};

serializable_struct!(RemoteModData {
  pub title: String,
  pub author: String,
  pub source_code: String,
  pub description: String,
  pub latest_version: ModDownload,
});

serializable_struct!(RemoteMod {
	pub common: CommonModData,
	pub data: RemoteModData,
});

pub type Map = HashMap<String, RemoteMod>;
