use std::{
	collections::HashSet,
	fs,
	path::PathBuf,
	string,
};

use async_trait::async_trait;
use lazy_regex::BytesRegex;
use steamlocate::SteamDir;

use super::provider::ProviderId;
use crate::{
	game_executable::OperatingSystem,
	game_mode::GameMode,
	installed_game::{
		self,
		InstalledGame,
	},
	owned_game::OwnedGame,
	provider::{
		ProviderActions,
		ProviderStatic,
	},
	steam::{
		appinfo::{
			self,
			SteamAppInfoFile,
		},
		id_lists,
		thumbnail::get_steam_thumbnail,
	},
	Result,
};

pub struct SteamProvider {
	steam_dir: SteamDir,
	app_info_file: SteamAppInfoFile,
}

impl ProviderStatic for SteamProvider {
	const ID: &'static ProviderId = &ProviderId::Steam;

	fn new() -> Result<Self>
	where
		Self: Sized,
	{
		let steam_dir = SteamDir::locate()?;
		let app_info_file = appinfo::read(steam_dir.path())?;

		Ok(Self {
			steam_dir,
			app_info_file,
		})
	}
}

#[async_trait]
impl ProviderActions for SteamProvider {
	fn get_installed_games(&self) -> Result<Vec<InstalledGame>> {
		let mut games: Vec<InstalledGame> = Vec::new();
		let mut used_paths: HashSet<PathBuf> = HashSet::new();
		let mut used_names: HashSet<String> = HashSet::new();

		for library in (self.steam_dir.libraries()?).flatten() {
			for app in library.apps().flatten() {
				if let Some(app_info) = self.app_info_file.apps.get(&app.app_id) {
					let sorted_launch_options = {
						let mut sorted_launch_options = app_info.launch_options.clone();
						sorted_launch_options.sort_by(|a, b| a.launch_id.cmp(&b.launch_id));
						sorted_launch_options
					};

					for launch_option in sorted_launch_options {
						if let Some(executable_path) = launch_option.executable.as_ref() {
							let full_path = &app.path.join(executable_path);

							if used_paths.contains(full_path) {
								continue;
							}

							if let Some(name) = &app.name {
								let discriminator = if used_names.contains(name) {
									launch_option.description.as_ref().map_or_else(
										|| {
											executable_path
												.to_str()
												.map(string::ToString::to_string)
										},
										|description| Some(description.clone()),
									)
								} else {
									None
								};

								if let Some(game) = installed_game::InstalledGame::new(
									full_path,
									name,
									*Self::ID,
									discriminator,
									Some(&launch_option),
									Some(get_steam_thumbnail(&app.app_id.to_string())),
								) {
									games.push(game);
									used_names.insert(name.clone());
									used_paths.insert(full_path.clone());
								}
							}
						}
					}
				}
			}
		}

		Ok(games)
	}

	async fn get_owned_games(&self) -> Result<Vec<OwnedGame>> {
		let steam_games = id_lists::get().await?;
		Ok(self
			.app_info_file
			.apps
			.iter()
			.filter_map(|(steam_id, app_info)| {
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

				// Games in appinfo.vdf aren't necessarily owned.
				// Most of them are, but there are also a bunch of other games that Steam needs to reference for one reason or another.
				// assets.vdf is another cache file, and from my (not very extensive) tests, it to really only include owned files.
				// Free games are some times not there though, so I'm presuming that any free game found in appinfo.vdf is owned.
				// appinfo.vdf is also still needed since most of the game data we want is there, so we can't just read everything from assets.vdf.
				let owned = app_info.is_free
					|| fs::read(
						self.steam_dir
							.path()
							.join("appcache/librarycache/assets.vdf"),
					)
					.map_or(false, |assets_cache_bytes| {
						// Would be smarter to actually parse assets.vdf and extract all the ids,
						// but I didn't feel like figuring out how to parse another binary vdf.
						// Maybe later. But most likely never.
						BytesRegex::new(&steam_id.to_string())
							.map_or(false, |regex| regex.is_match(&assets_cache_bytes))
					});

				if !owned {
					return None;
				}

				let installed = self
					.steam_dir
					.app(*steam_id)
					.map_or(false, |steam_app| steam_app.is_some());

				let release_date = app_info
					.original_release_date
					.or(app_info.steam_release_date)
					.unwrap_or_default();

				let game_mode = if app_info
					.launch_options
					.iter()
					.any(|launch| launch.get_game_mode() == GameMode::VR)
				{
					GameMode::VR
				} else {
					GameMode::Flat
				};

				let steam_game = steam_games.get(&steam_id.to_string());

				Some(OwnedGame {
					id: steam_id.to_string(),
					provider_id: *Self::ID,
					name: app_info.name.clone(),
					installed,
					os_list,
					engine: steam_game.map(|game| game.engine),
					release_date,
					thumbnail_url: get_steam_thumbnail(&steam_id.to_string()),
					game_mode: Some(game_mode),
					uevr_score: steam_game.and_then(|game| game.uevr_score),
				})
			})
			.collect())
	}
}
