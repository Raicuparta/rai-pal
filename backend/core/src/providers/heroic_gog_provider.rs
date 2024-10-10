#![cfg(target_os = "linux")]

use std::{
	fmt::Debug,
	fs::{self, read_to_string},
	io::{self},
	path::Path,
};

use crate::{
	installed_game::InstalledGame,
	owned_game::OwnedGame,
	providers::provider::{ProviderActions, ProviderId, ProviderStatic},
	result::Result as GameResult,
};

use directories::BaseDirs;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct Installed {
	platform: String,
	executable: String,
	install_path: String,
	install_size: String,
	is_dlc: bool,
	version: String,
	#[serde(rename(deserialize = "appName"))]
	app_name: String,
	#[serde(rename(deserialize = "installedDLCs"))]
	installed_dlcs: Vec<String>,
	language: String,
	#[serde(rename(deserialize = "versionEtag"))]
	version_etag: String,
	#[serde(rename(deserialize = "buildId"))]
	build_id: String,
	#[serde(rename(deserialize = "pinnedVersion"))]
	pinned_version: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct RootInstalled {
	installed: Vec<Installed>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Game {
	app_name: String,
	runner: String,
	title: String,
	#[serde(rename(deserialize = "canRunOffline"))]
	can_run_offline: bool,
	install: Install,
	is_installed: bool,
	art_cover: String,
	art_square: String,
	art_background: Option<String>,
	cloud_save_enabled: Option<bool>,
	art_icon: Option<String>,
	extra: Option<Extra>,
	folder_name: Option<String>,
	save_folder: Option<String>,
	is_mac_native: Option<bool>,
	is_linux_native: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Install {
	is_dlc: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct Extra {
	about: About,
	reqs: Vec<String>,
	genres: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct About {
	description: String,
	#[serde(rename(deserialize = "shortDescription"))]
	short_description: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Timestamp {
	games: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Root {
	games: Vec<Game>,
}

fn get_detected_games() -> Result<Vec<Game>, io::Error> {
	let dirs = BaseDirs::new().expect("Failed to get user directories");
	let config_dir = dirs.config_dir();
	let file_content =
		read_to_string(Path::new(&config_dir).join("heroic/store_cache/gog_library.json"))?;
	let installed_games = read_installed_games()?;

	Ok(serde_json::from_str::<Root>(file_content.as_str())?
		.games
		.into_iter()
		// gog-redist is not a game but it shows up in the library
		.filter(|game| game.app_name != "gog-redist".to_string())
		.map(|mut game| {
			if installed_games.contains(&game.app_name) {
				// is_installed props from the library are not reliable
				game.is_installed = true;
			}
			game
		})
		.collect())
}

fn read_installed_games() -> Result<Vec<String>, io::Error> {
	let dirs = BaseDirs::new().expect("Failed to get user directories");
	let config_dir = dirs.config_dir();
	let file_content =
		read_to_string(Path::new(&config_dir).join("heroic/gog_store/installed.json"))?;

	Ok(
		serde_json::from_str::<RootInstalled>(file_content.as_str())?
			.installed
			.into_iter()
			.map(|game| game.app_name)
			.collect(),
	)
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct PlayTask {
	category: String,
	is_primary: Option<bool>,
	languages: Vec<String>,
	name: String,
	path: Option<String>,
	type_: String,
	link: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct GogGame {
	build_id: String,
	client_id: String,
	game_id: String,
	language: String,
	languages: Vec<String>,
	name: String,
	play_tasks: Vec<PlayTask>,
	root_game_id: String,
	version: i32,
}

fn read_info_file(path: &Path, app_id: &str) -> Result<GogGame, String> {
	let file_path = path.join(format!("goggame-{}.info", app_id));

	match fs::read_to_string(file_path) {
		Ok(json_string) => match serde_json::from_str(&json_string) {
			Ok(json_value) => Ok(json_value),
			Err(err) => Err(format!("Error parsing JSON: {}", err)),
		},
		Err(err) => Err(format!("Error reading file: {}", err)),
	}
}

#[derive(Clone)]
pub struct HeroicGog {}

impl HeroicGog {
	fn get_installed_game(entry: &Game) -> Option<InstalledGame> {
		let dirs = BaseDirs::new().expect("Failed to get user directories");
		let home_dir = dirs.home_dir();
		let game_path = Path::new(&home_dir)
			.join("Games/Heroic")
			.join(&entry.folder_name.clone().unwrap());
		let infos = read_info_file(game_path.as_path(), &entry.app_name).unwrap();

		let executable_name = infos.play_tasks.iter().find_map(|task| {
			if task.is_primary.unwrap_or(false) {
				task.path.clone()
			} else {
				None
			}
		});

		let mut game = InstalledGame::new(
			game_path.join(executable_name.unwrap()).as_path(),
			&entry.title,
			Self::ID.to_owned(),
		)?;

		game.set_start_command_string(&get_start_command("gog", &entry.app_name));
		game.set_provider_game_id(&entry.app_name);

		game.set_thumbnail_url(&entry.art_cover);

		Some(game)
	}
}

impl ProviderStatic for HeroicGog {
	const ID: &'static ProviderId = &ProviderId::HeroicGog;

	fn new() -> GameResult<Self>
	where
		Self: Sized,
	{
		Ok(Self {})
	}
}

impl ProviderActions for HeroicGog {
	async fn get_games<TInstalledCallback, TOwnedCallback>(
		&self,
		mut installed_callback: TInstalledCallback,
		mut owned_callback: TOwnedCallback,
	) -> GameResult
	where
		TInstalledCallback: FnMut(InstalledGame) + Send + Sync,
		TOwnedCallback: FnMut(OwnedGame) + Send + Sync,
	{
		let games = get_detected_games()?;
		for game in games {
			let mut owned_game = OwnedGame::new(&game.app_name, *Self::ID, &game.title);
			owned_game.set_thumbnail_url(&game.art_cover);
			owned_callback(owned_game);

			if game.is_installed {
				if let Some(installed_game) = HeroicGog::get_installed_game(&game) {
					installed_callback(installed_game);
				}
			}
		}

		Ok(())
	}
}

pub fn get_start_command(source: &str, app_id: &str) -> String {
	format!("heroic://launch/{source}/{app_id}")
}

#[cfg(test)]
mod tests {

	use super::*;

	#[test]
	fn test_heroic_gog_launcher() -> Result<(), io::Error> {
		let games = get_detected_games()?;

		println!("Games: {games:#?}");

		assert_eq!(games.len(), 2);

		assert_eq!(games[0].title, "RollerCoaster Tycoon 2 Triple Thrill Pack");

		// assert!(games[0].path_game_dir.is_some());

		// assert!(games[0].path_box_art.is_none());

		Ok(())
	}
}
