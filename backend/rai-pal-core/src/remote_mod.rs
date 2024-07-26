use std::collections::HashMap;

use rai_pal_proc_macros::serializable_struct;

use crate::{game_mod::CommonModData, mod_loaders::mod_database::ModDownload};

#[serializable_struct]
pub struct RemoteModData {
	pub title: String,
	pub deprecated: bool,
	pub author: String,
	pub source_code: String,
	pub description: String,
	pub latest_version: Option<ModDownload>,
}

#[serializable_struct]
pub struct RemoteMod {
	pub common: CommonModData,
	pub data: RemoteModData,
}

pub type Map = HashMap<String, RemoteMod>;
