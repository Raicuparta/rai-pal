use std::collections::HashMap;

use async_trait::async_trait;
use enum_dispatch::enum_dispatch;

use crate::{
	installed_game::InstalledGame,
	owned_game::OwnedGame,
	providers::{
		manual_provider::ManualProvider,
		steam_provider::SteamProvider,
	},
	serializable_enum,
	Error,
	Result,
};

serializable_enum!(ProviderId { Steam, Manual });

#[enum_dispatch]
pub enum Provider {
	SteamProvider,
	ManualProvider,
}

#[async_trait]
#[enum_dispatch(Provider)]
pub trait ProviderActions {
	fn get_installed_games(&self) -> Result<Vec<InstalledGame>>;

	async fn get_owned_games(&self) -> Result<Vec<OwnedGame>>;
}

pub trait ProviderStatic {
	const ID: &'static ProviderId;

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

fn add_entry<TProvider: ProviderActions + ProviderStatic, F>(map: &mut Map, error_handler: &F)
where
	Provider: std::convert::From<TProvider>,
	F: Fn(Error) + Send,
{
	match create_map_entry::<TProvider>() {
		Ok((key, value)) => {
			map.insert(key, value);
		}
		Err(error) => error_handler(error),
	}
}

pub fn get_map<F>(error_handler: F) -> Map
where
	F: Fn(Error) + Send,
{
	let mut map = Map::new();

	add_entry::<SteamProvider, F>(&mut map, &error_handler);
	add_entry::<ManualProvider, F>(&mut map, &error_handler);

	map
}
