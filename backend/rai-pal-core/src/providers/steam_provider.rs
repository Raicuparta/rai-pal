use std::{
	collections::{HashMap, HashSet},
	fs,
	marker::{Send, Sync},
	path::{Path, PathBuf},
};

use async_trait::async_trait;
use futures::StreamExt;
use lazy_regex::BytesRegex;
use steamlocate::SteamDir;

use super::{
	provider::ProviderId,
	provider_command::{ProviderCommand, ProviderCommandAction},
};
use crate::{
	app_type::AppType,
	game_engines::game_engine::GameEngine,
	game_executable::OperatingSystem,
	game_mode::GameMode,
	installed_game::{self, InstalledGame},
	owned_game::OwnedGame,
	pc_gaming_wiki,
	providers::provider::{ProviderActions, ProviderStatic},
	remote_game::{self, RemoteGame},
	result::Result,
	steam::{
		appinfo::{SteamAppInfo, SteamAppInfoReader, SteamLaunchOption},
		id_lists,
		thumbnail::get_steam_thumbnail,
	},
};

#[derive(Clone)]
pub struct Steam {
	remote_game_cache: remote_game::Map,
}

impl ProviderStatic for Steam {
	const ID: &'static ProviderId = &ProviderId::Steam;

	fn new() -> Result<Self>
	where
		Self: Sized,
	{
		let remote_game_cache = Self::try_get_remote_game_cache();

		Ok(Self { remote_game_cache })
	}
}

impl Steam {
	pub fn get_owned_game(
		app_info: &SteamAppInfo,
		ids_whitelist: &HashSet<String>,
	) -> Option<OwnedGame> {
		let mut game = OwnedGame::new(&app_info.app_id.to_string(), *Self::ID, &app_info.name);

		let id_string = app_info.app_id.to_string();

		let owned =
			app_info.is_free || ids_whitelist.is_empty() || ids_whitelist.contains(&id_string);

		if !owned {
			return None;
		}

		let os_list: HashSet<_> = app_info
			.launch_options
			.iter()
			.filter_map(|launch| {
				launch
					.os_list
					.as_ref()
					.and_then(|os_list| match os_list.as_str() {
						"linux" => Some(OperatingSystem::Linux),
						"windows" => Some(OperatingSystem::Windows),
						_ => None,
					})
			})
			.collect();

		let game_mode = if app_info
			.launch_options
			.iter()
			.any(|launch| launch.get_game_mode() == GameMode::VR)
		{
			GameMode::VR
		} else {
			GameMode::Flat
		};

		game.set_thumbnail_url(&get_steam_thumbnail(&id_string))
			.set_os_list(os_list)
			.set_game_mode(game_mode)
			.add_provider_command(
				ProviderCommandAction::ShowInLibrary,
				ProviderCommand::String(format!("steam://nav/games/details/{id_string}")),
			)
			.add_provider_command(
				ProviderCommandAction::ShowInStore,
				ProviderCommand::String(format!("steam://store/{id_string}")),
			)
			.add_provider_command(
				ProviderCommandAction::Install,
				ProviderCommand::String(format!("steam://install/{id_string}")),
			)
			.add_provider_command(
				ProviderCommandAction::OpenInBrowser,
				ProviderCommand::String(format!("https://store.steampowered.com/app/{id_string}")),
			);

		if let Some(release_date) = app_info
			.original_release_date
			.or(app_info.steam_release_date)
		{
			game.set_release_date(release_date.into());
		}

		if let Some(app_type) = &app_info.app_type {
			if app_type == "Game" {
				game.set_app_type(AppType::Game);
			} else if app_type == "Demo" {
				game.set_app_type(AppType::Demo);
			}
		} else {
			// We only try to guess the app type if couldn't read it from appinfo.
			// For instance, something marked as Tool or Application shouldn't be marked as a game.
			game.guess_app_type();
		}

		Some(game)
	}

	pub fn get_installed_games(
		app_info: &SteamAppInfo,
		dir_app: &steamlocate::SteamApp,
	) -> Vec<InstalledGame> {
		let mut used_paths: HashSet<PathBuf> = HashSet::new();
		let mut used_names: HashSet<String> = HashSet::new();
		let mut installed_games = Vec::new();

		let sorted_launch_options = {
			let mut sorted_launch_options = app_info.launch_options.clone();
			sorted_launch_options.sort_by(|a, b| a.launch_id.cmp(&b.launch_id));
			sorted_launch_options
		};

		for launch_option in sorted_launch_options {
			if let Some(executable_path) = launch_option.executable.as_ref() {
				let full_path = &dir_app.path.join(executable_path);

				if used_paths.contains(full_path) {
					continue;
				}

				if let Some(name) = &dir_app.name {
					if let Some(mut game) =
						installed_game::InstalledGame::new(full_path, name, *Self::ID)
					{
						let discriminator_option = if used_names.contains(name) {
							Some(launch_option.description.as_ref().map_or_else(
								|| executable_path.display().to_string(),
								Clone::clone,
							))
						} else {
							None
						};

						if let Some(discriminator) = &discriminator_option {
							game.set_discriminator(discriminator);
						}

						let app_id_string = dir_app.app_id.to_string();

						game.set_provider_game_id(&app_id_string);
						game.set_thumbnail_url(&get_steam_thumbnail(&app_id_string));
						game.set_start_command_string(&get_start_command(
							&launch_option,
							&discriminator_option,
						));

						used_names.insert(name.clone());
						used_paths.insert(full_path.clone());
						installed_games.push(game);
					}
				}
			}
		}

		installed_games
	}

	pub async fn get_remote_game(
		&self,
		app_info: SteamAppInfo,
		steam_games: &HashMap<String, id_lists::SteamGame>,
	) -> Option<RemoteGame> {
		let id_string = app_info.app_id.to_string();
		let mut remote_game = RemoteGame::new(*Self::ID, &id_string);

		if let Some(cached_remote_game) = self.remote_game_cache.get(&remote_game.id) {
			return Some(cached_remote_game.clone());
		}

		let steam_game_option = steam_games.get(&id_string);

		match pc_gaming_wiki::get_engine(&format!("Steam_AppID HOLDS \"{id_string}\"")).await {
			Ok(Some(pc_gaming_wiki_engine)) => {
				remote_game.set_engine(pc_gaming_wiki_engine);
			}
			Ok(None) => {
				if let Some(engine_brand) = steam_game_option.map(|steam_game| steam_game.engine) {
					remote_game.set_engine(GameEngine {
						brand: engine_brand,
						version: None,
					});
				}
			}
			Err(_) => {
				remote_game.set_skip_cache(true);
			}
		}

		Some(remote_game)
	}

	fn get_owned_ids_whitelist(steam_path: &Path) -> Result<HashSet<String>> {
		// Games in appinfo.vdf aren't necessarily owned.
		// Most of them are, but there are also a bunch of other games that Steam needs to reference for one reason or another.
		// assets.vdf is another cache file, and from my (not very extensive) tests, it does really only include owned files.
		// Free games are some times not there though, so later in the code I'm presuming that any free game found in appinfo.vdf is owned.
		// appinfo.vdf is also still needed since most of the game data we want is there, so we can't just read everything from assets.vdf.
		let assets_cache_string = fs::read(steam_path.join("appcache/librarycache/assets.vdf"))?;

		// This file has a bunch of ids, and they're always just numbers surrounded by zeros.
		// We could have a smarter parse (this is a binary vdf), but let's just do this for now (probably forever).
		let isolated_numbers: HashSet<String> = BytesRegex::new(r"\x00(\d+)\x00")?
			.captures_iter(&assets_cache_string)
			.filter_map(|captures| {
				captures.get(1).and_then(|capture_match| {
					String::from_utf8(capture_match.as_bytes().to_owned()).ok()
				})
			})
			.collect();

		Ok(isolated_numbers)
	}

	async fn get_steam_games<TInstalledCallback, TOwnedCallback, TRemoteCallback>(
		&self,
		installed_callback: TInstalledCallback,
		owned_callback: TOwnedCallback,
		remote_callback: TRemoteCallback,
	) -> Result
	where
		TInstalledCallback: Fn(InstalledGame) + Send + Sync,
		TOwnedCallback: Fn(OwnedGame) + Send + Sync,
		TRemoteCallback: Fn(RemoteGame) + Send + Sync,
	{
		let steam_dir = SteamDir::locate()?;
		let app_info_reader = SteamAppInfoReader::new(steam_dir.path())?;

		let mut steam_dir_app_map: HashMap<_, _> = HashMap::new();
		for library in (steam_dir.libraries()?).flatten() {
			for app in library.apps().flatten() {
				steam_dir_app_map.insert(app.app_id, app);
			}
		}

		let owned_ids_whitelist =
			Self::get_owned_ids_whitelist(steam_dir.path()).unwrap_or_else(|err| {
				log::error!("Failed to read Steam assets.vdf: {}", err);
				HashSet::new()
			});

		let steam_games = id_lists::get().await?;

		futures::stream::iter(app_info_reader)
			.for_each_concurrent(Some(20), |app_info_result| {
				async {
					match app_info_result {
						Ok(app_info) => {
							if let Some(owned_game) =
								Self::get_owned_game(&app_info, &owned_ids_whitelist)
							{
								owned_callback(owned_game);

								if let Some(remote_game) =
									self.get_remote_game(app_info.clone(), &steam_games).await
								{
									remote_callback(remote_game);
									// TODO: cache
									// self.remote_game_cache
									// 	.insert(remote_game.id.clone(), remote_game.clone());
								}
							}
							if let Some(dir_app) = steam_dir_app_map.get(&app_info.app_id) {
								for installed_game in Self::get_installed_games(&app_info, dir_app)
								{
									installed_callback(installed_game);
								}
							}
						}
						Err(error) => {
							log::error!("Failed to read Steam appinfo: {}", error);
						}
					}
				}
			})
			.await;

		Ok(())
	}
}

#[async_trait]
impl ProviderActions for Steam {
	async fn get_games<TInstalledCallback, TOwnedCallback, TRemoteCallback>(
		&self,
		installed_callback: TInstalledCallback,
		owned_callback: TOwnedCallback,
		remote_callback: TRemoteCallback,
	) -> Result
	where
		TInstalledCallback: Fn(InstalledGame) + Send + Sync,
		TOwnedCallback: Fn(OwnedGame) + Send + Sync,
		TRemoteCallback: Fn(RemoteGame) + Send + Sync,
	{
		self.get_steam_games(installed_callback, owned_callback, remote_callback)
			.await
	}
}

pub fn get_start_command(
	steam_launch: &SteamLaunchOption,
	discriminator: &Option<String>,
) -> String {
	if discriminator.is_none() {
		// If a game has no discriminator, it means we're probably using the default launch option.
		// For those, we use the steam://rungameid command, since that one will make steam show a nice
		// loading popup, wait for game updates, etc.

		format!("steam://rungameid/{}", steam_launch.app_id)
	} else {
		// For the few cases where we're showing an alternative launch option, we use the steam://launch command.
		// This one will show an error if the game needs an update, and doesn't show the nice loading popup,
		// but it allows us to specify the specific launch option to run.
		// This one also supports passing "dialog" instead of the app_type, (steam://launch/{app_id}/dialog)
		// which makes Steam show the launch selection dialog, but that dialog stops showing if the user
		// selects the "don't ask again" checkbox.
		format!(
			"steam://launch/{}/{}",
			steam_launch.app_id,
			steam_launch.launch_type.as_deref().unwrap_or(""),
		)
	}
}
