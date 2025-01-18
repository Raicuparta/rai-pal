use std::{
	collections::{HashMap, HashSet},
	marker::{Send, Sync},
	path::{Path, PathBuf},
};

use steamlocate::SteamDir;

use super::{
	provider::ProviderId,
	provider_command::{ProviderCommand, ProviderCommandAction},
};
use crate::{
	game::{Game, GameId},
	game_tag::GameTag,
	installed_game::{self, InstalledGame},
	providers::provider::{ProviderActions, ProviderStatic},
	result::Result,
	steam::{
		appinfo::{self, SteamAppInfo, SteamAppInfoReader, SteamLaunchOption},
		thumbnail::get_steam_thumbnail,
	},
};

#[derive(Clone)]
pub struct Steam;

impl ProviderStatic for Steam {
	const ID: &'static ProviderId = &ProviderId::Steam;

	fn new() -> Result<Self>
	where
		Self: Sized,
	{
		Ok(Self {})
	}
}

impl Steam {
	pub fn get_installed_games(app_info: &SteamAppInfo, app_path: &Path) -> Vec<InstalledGame> {
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
				let full_path = &app_path.join(executable_path);

				if used_paths.contains(full_path) {
					continue;
				}

				let app_name = app_info.name.clone();

				if let Some(mut game) = installed_game::InstalledGame::new(full_path) {
					let discriminator_option =
						if used_names.contains(&app_name) {
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

					game.set_start_command_string(&get_start_command(
						&launch_option,
						&discriminator_option,
					));

					used_names.insert(app_name);
					used_paths.insert(full_path.clone());
					installed_games.push(game);
				}
			}
		}

		installed_games
	}

	fn get_owned_ids_whitelist(steam_path: &Path) -> Result<HashSet<String>> {
		// Games in appinfo.vdf aren't necessarily owned.
		// Most of them are, but there are also a bunch of other games that Steam needs to reference for one reason or another.
		// packageinfo.vdf is another cache file, and from my (not very extensive) tests, it does really only include owned packages.
		// appinfo.vdf is also still needed since most of the game data we want is there.

		let package_info = appinfo::PackageInfo::read(&steam_path.join("appcache/packageinfo.vdf"))?;

		Ok(package_info.get_app_ids())
	}
}

impl ProviderActions for Steam {
	async fn get_games<TCallback>(&self, mut callback: TCallback) -> Result
	where
		TCallback: FnMut(Game) + Send + Sync,
	{
		let steam_dir = SteamDir::locate()?;
		let steam_path = steam_dir.path();
		let app_info_reader = SteamAppInfoReader::new(&appinfo::get_path(steam_path))?;
		let mut app_paths = HashMap::<u32, PathBuf>::new();
		for library in (steam_dir.libraries()?).flatten() {
			for app in library.apps().flatten() {
				app_paths.insert(app.app_id, library.resolve_app_dir(&app));
			}
		}

		let owned_ids_whitelist = Self::get_owned_ids_whitelist(steam_path).unwrap_or_else(|err| {
			log::error!("Failed to read Steam assets cache: {}", err);
			HashSet::new()
		});

		log::info!("whitelist size: {}", owned_ids_whitelist.len());

		for app_info_result in app_info_reader {
			match app_info_result {
				Ok(app_info) => {
					let external_id = app_info.app_id.to_string();

					if !app_info.is_free
						&& !owned_ids_whitelist.is_empty()
						&& !owned_ids_whitelist.contains(&external_id)
					{
						continue;
					}

					let installed_games = app_paths
						.get(&app_info.app_id)
						.map(|app_path| Self::get_installed_games(&app_info, app_path))
						.unwrap_or_default();

					let mut game = Game::new(
						GameId {
							game_id: external_id.clone(),
							provider_id: *Self::ID,
						},
						&app_info.name,
					);

					game.set_thumbnail_url(&get_steam_thumbnail(&external_id))
						.add_provider_command(
							ProviderCommandAction::ShowInLibrary,
							ProviderCommand::String(format!(
								"steam://nav/games/details/{external_id}"
							)),
						)
						.add_provider_command(
							ProviderCommandAction::ShowInStore,
							ProviderCommand::String(format!("steam://store/{external_id}")),
						)
						.add_provider_command(
							ProviderCommandAction::Install,
							ProviderCommand::String(format!("steam://install/{external_id}")),
						)
						.add_provider_command(
							ProviderCommandAction::OpenInBrowser,
							ProviderCommand::String(format!(
								"https://store.steampowered.com/app/{external_id}"
							)),
						);

					if app_info
						.launch_options
						.iter()
						.any(appinfo::SteamLaunchOption::is_vr)
					{
						game.add_tag(GameTag::VR);
					}

					if let Some(release_date) = app_info
						.original_release_date
						.or(app_info.steam_release_date)
					{
						game.set_release_date(release_date.into());
					}

					if let Some(app_type) = &app_info.app_type {
						if app_type == "Demo" {
							game.add_tag(GameTag::Demo);
						}
					}

					if installed_games.is_empty() {
						callback(game);
					} else {
						for installed_game in installed_games {
							let mut game_with_installed = game.clone();
							game_with_installed.id = GameId {
								game_id: format!(
									"{}_{}",
									&game_with_installed.external_id, &installed_game.id
								),
								provider_id: *Self::ID,
							};
							game_with_installed.installed_game = Some(installed_game);
							callback(game_with_installed);
						}
					}
				}
				Err(error) => {
					log::error!("Failed to read Steam appinfo: {}", error);
				}
			}
		}

		Ok(())
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
