use std::{
	collections::HashMap,
	time::Instant,
};

use async_trait::async_trait;
use enum_dispatch::enum_dispatch;

use super::{
	epic_provider::EpicProvider,
	gog_provider::GogProvider,
	xbox_provider::XboxProvider,
};
use crate::{
	debug::LoggableInstant,
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

serializable_enum!(ProviderId {
	Steam,
	Manual,
	Epic,
	Gog,
	Xbox,
});

#[enum_dispatch]
pub enum Provider {
	SteamProvider,
	ManualProvider,
	EpicProvider,
	GogProvider,
	XboxProvider,
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
	let now = &mut Instant::now();

	add_entry::<SteamProvider, F>(&mut map, &error_handler);
	now.log_next("set up provider (Steam)");
	add_entry::<EpicProvider, F>(&mut map, &error_handler);
	now.log_next("set up provider (Epic)");
	add_entry::<GogProvider, F>(&mut map, &error_handler);
	now.log_next("set up provider (Gog)");
	add_entry::<XboxProvider, F>(&mut map, &error_handler);
	now.log_next("set up provider (Xbox)");
	add_entry::<ManualProvider, F>(&mut map, &error_handler);
	now.log_next("set up provider (Manual)");

	map
}
