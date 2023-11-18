use std::collections::HashMap;

use enum_dispatch::enum_dispatch;

use super::{
	manual_provider::ManualProvider,
	steam_provider::SteamProvider,
};
use crate::{
	installed_game,
	mod_loaders::mod_loader,
	Result,
};

#[enum_dispatch]
pub enum Provider {
	SteamProvider,
	ManualProvider,
}

#[enum_dispatch(Provider)]
pub trait ProviderActions {
	fn get_installed_games(&self, mod_loaders: &mod_loader::DataMap)
		-> Result<installed_game::Map>;
}

pub trait ProviderStatic {
	const ID: &'static str;

	fn new() -> Result<Self>
	where
		Self: Sized;
}

type Map = HashMap<String, Provider>;

fn create_map_entry<TProvider: ProviderActions + ProviderStatic>() -> Result<(String, Provider)>
where
	Provider: std::convert::From<TProvider>,
{
	let mod_loader: Provider = TProvider::new()?.into();

	Ok((TProvider::ID.to_string(), mod_loader))
}

fn add_entry<TProvider: ProviderActions + ProviderStatic>(map: &mut Map)
where
	Provider: std::convert::From<TProvider>,
{
	match create_map_entry::<TProvider>() {
		Ok((key, value)) => {
			map.insert(key, value);
		}
		Err(err) => println!("Failed to create map entry: {err}"),
	}
}

pub fn get_map() -> Map {
	let mut map = Map::new();

	add_entry::<SteamProvider>(&mut map);
	add_entry::<ManualProvider>(&mut map);

	map
}
