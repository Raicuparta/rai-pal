use std::{collections::HashMap, path::PathBuf, process::Command};

use chrono::DateTime;
use log::error;
use rai_pal_proc_macros::serializable_struct;
use rusqlite::{Connection, OpenFlags};

use super::provider_command::{ProviderCommand, ProviderCommandAction};
use crate::{
	app_paths,
	game::DbGame,
	game_providers::game_provider::{GameProviderId, ProviderActions, WineProviderActions},
	local_database::{app_database::DbMutex, game_database::GameDatabase},
	result::{Error, LogErrExt, Result},
	wine,
};

#[derive(Clone)]
pub struct Itch {}

impl Itch {
	fn get_exe_path(cave: &ItchDatabaseCave) -> Option<PathBuf> {
		let verdict = cave.verdict.as_ref()?;

		if let Some(candidates) = &verdict.candidates
			&& let Some(candidate) = candidates.first()
		{
			return Some(verdict.base_path.join(&candidate.path));
		}

		// Fallback: scan base_path for executables if butler hasn't cached candidates
		Self::scan_for_exe(&verdict.base_path)
	}

	fn scan_for_exe(base_path: &std::path::Path) -> Option<PathBuf> {
		use std::fs;

		let mut entries: Vec<_> = fs::read_dir(base_path)
			.ok_or_log("Failed to read Itch dir")?
			.filter_map(|e| e.ok_or_log("Failed to read Itch dir entry"))
			.filter(|e| e.file_name() != ".itch")
			.collect();

		// If there's only one entry and it's a directory, look inside it
		if entries.len() == 1 && entries[0].file_type().is_ok_and(|ft| ft.is_dir()) {
			entries = fs::read_dir(entries[0].path())
				.ok_or_log("Failed to read Itch dir")?
				.filter_map(|e| e.ok_or_log("Failed to read Itch dir entry"))
				.collect();
		}

		for entry in entries {
			let path = entry.path();
			if let Some(ext) = path.extension().and_then(|e| e.to_str())
				&& (ext == "exe" || ext == "x86" || ext == "x86_64")
			{
				return Some(path);
			}
		}

		None
	}

	fn get_game(row: &ItchDatabaseGame) -> DbGame {
		let mut game = DbGame::new(GameProviderId::Itch, row.id.to_string(), row.title.clone());

		game.thumbnail_url.clone_from(&row.cover_url);

		if let Some(date_time) = row.published_at.as_ref().and_then(|published_at| {
			DateTime::parse_from_rfc3339(published_at).ok_or_log(&format!(
				"Failed to parse itch published_at `{published_at}` for `{}`",
				row.title
			))
		}) {
			game.release_date_rfc3339 = Some(date_time.to_rfc3339());
		}

		game.add_provider_command(
			ProviderCommandAction::ShowInLibrary,
			ProviderCommand::String(format!("itch://games/{}", row.id)),
		)
		.add_provider_command(
			ProviderCommandAction::Install,
			ProviderCommand::String(format!("itch://install?game_id={}", row.id)),
		);

		game
	}
}

#[serializable_struct]
pub struct ItchDatabaseGame {
	id: i32,
	title: String,
	url: Option<String>,
	published_at: Option<String>,
	cover_url: Option<String>,
}

#[serializable_struct]
pub struct ItchDatabaseCave {
	id: i32,
	verdict: Option<ItchDatabaseVerdict>,
	title: String,
	cover_url: Option<String>,
}

#[serializable_struct]
pub struct ItchDatabaseVerdict {
	base_path: PathBuf,
	candidates: Option<Vec<ItchDatabaseCandidate>>,
}

#[serializable_struct]
pub struct ItchDatabaseCandidate {
	path: PathBuf,
}

#[serializable_struct]
pub struct ItchDatabase {
	games: Vec<ItchDatabaseGame>,
	caves: Vec<ItchDatabaseCave>,
}

impl ProviderActions for Itch {
	fn insert_games(&self, db: &DbMutex) -> Result {
		let Some(database) = get_database()? else {
			log::info!(
				"Itch database file not found. Probably means user hasn't installed the Itch app."
			);
			return Ok(());
		};
		let caves_map: HashMap<_, _> = database
			.caves
			.into_iter()
			.map(|cave| (cave.id, cave))
			.collect();

		for db_entry in database.games {
			let mut game = Self::get_game(&db_entry);
			if let Some(exe_path) = caves_map.get(&db_entry.id).and_then(Self::get_exe_path) {
				game.set_executable(&exe_path);
			}
			db.insert_game(&game);
		}

		Ok(())
	}
}

impl WineProviderActions for Itch {
	fn get_wine_prefix_path(&self, _game: &DbGame) -> Result<PathBuf> {
		let prefix = get_itch_wine_prefix()?;
		log::info!("Resolved Itch wine prefix: `{}`", prefix.display());
		Ok(prefix)
	}

	fn get_wine_binary_path(&self, _game: &DbGame) -> Result<PathBuf> {
		Ok(find_itch_wine())
	}

	fn get_run_with_wine_command(&self, game: &DbGame) -> Result<Command> {
		let wine_prefix_path = self.get_wine_prefix_path(game)?;
		let wine_binary = self.get_wine_binary_path(game)?;

		let mut cmd = Command::new(&wine_binary);
		cmd.env("WINEPREFIX", &wine_prefix_path);

		// Flatpak-bundled wine needs WINESERVER set explicitly to find its wineserver binary.
		if let Some(wineserver) = wine_binary.parent().map(|p| p.join("wineserver"))
			&& wineserver.exists()
		{
			cmd.env("WINESERVER", &wineserver);
		}

		Ok(cmd)
	}

	fn set_wine_dll_overrides(&self, game: &DbGame, dll_overrides: &[String]) -> Result {
		let prefix_path = self.get_wine_prefix_path(game)?;
		wine::set_wine_dll_overrides_in_reg(&prefix_path, dll_overrides)
	}
}

fn find_itch_wine() -> PathBuf {
	let wine_name = "wine";

	let flatpak_wine =
		PathBuf::from("/var/lib/flatpak/app/io.itch.itch/current/active/files/bin/wine");

	if flatpak_wine.exists() {
		log::info!("Found itch flatpak wine: `{}`", flatpak_wine.display());
		return flatpak_wine;
	}

	if let Some(path_var) = std::env::var_os("PATH") {
		for dir in std::env::split_paths(&path_var) {
			let wine_bin = dir.join(wine_name);
			if wine_bin.exists() {
				log::info!("Found wine via PATH: `{}`", wine_bin.display());
				return wine_bin;
			}
		}
	}

	log::warn!("Could not find `wine` on PATH or in itch flatpak. Falling back to bare name.");
	PathBuf::from(wine_name)
}

fn get_itch_wine_prefix() -> Result<PathBuf> {
	let base_dirs = app_paths::base_dirs()?;

	let candidates = [
		base_dirs.home_dir().join(".var/app/io.itch.itch/data/wine"),
		base_dirs.home_dir().join(".itch/wine"),
	];

	for candidate in &candidates {
		if candidate.join("drive_c").exists() {
			return Ok(candidate.clone());
		}
	}

	Err(Error::Itch(format!(
		"Itch wine prefix not found. Tried:\n{}",
		candidates
			.iter()
			.map(|p| format!("  - {}", p.display()))
			.collect::<Vec<_>>()
			.join("\n")
	)))
}

fn parse_verdict(json_option: Option<&String>) -> Option<ItchDatabaseVerdict> {
	let json = json_option?;
	match serde_json::from_str(json) {
		Ok(verdict) => Some(verdict),
		Err(err) => {
			error!("Failed to parse verdict from json `{json}`. Error: {err}");
			None
		}
	}
}

fn find_butler_db() -> Result<Option<PathBuf>> {
	let base_dirs = app_paths::base_dirs()?;

	Ok([
		base_dirs.config_dir().join("itch"),
		base_dirs
			.home_dir()
			.join(".var/app/io.itch.itch/config/itch"),
	]
	.into_iter()
	.find_map(|p| {
		let db = p.join("db").join("butler.db");
		db.is_file().then_some(db)
	}))
}

fn get_database() -> Result<Option<ItchDatabase>> {
	let Some(db_path) = find_butler_db()? else {
		return Ok(None);
	};
	let db_display = db_path.display().to_string();
	let connection = Connection::open_with_flags(db_path, OpenFlags::SQLITE_OPEN_READ_ONLY)?;

	let mut caves_statement = connection.prepare(
		r"SELECT
			caves.game_id, caves.verdict, games.title, games.cover_url
    FROM
			caves
    JOIN
			games ON caves.game_id = games.id;
  ",
	)?;
	let cave_rows = caves_statement.query_map([], |row| {
		Ok(ItchDatabaseCave {
			id: row.get("game_id")?,
			title: row.get("title")?,
			verdict: parse_verdict(
				row.get("verdict")
					.ok_or_log(&format!(
						"Failed to parse itch cave verdict for {db_display}"
					))
					.as_ref(),
			),
			cover_url: row.get("cover_url").ok_or_log(&format!(
				"Failed to parse itch cave cover_url for {db_display}"
			)),
		})
	})?;

	let mut games_statement = connection.prepare(
		r"SELECT
			id, title, url, published_at, cover_url
		FROM
			'games'
		WHERE
			type='default' AND classification='game'
		",
	)?;
	let game_rows = games_statement.query_map([], |row| {
		Ok(ItchDatabaseGame {
			id: row.get(0)?,
			title: row.get(1)?,
			url: row.get(2).ok_or_log("Failed to read url on itch row"),
			published_at: row
				.get(3)
				.ok_or_log("Failed to read published_at on itch row"),
			cover_url: row.get(4).ok_or_log("Failed to read cover_url on itch row"),
		})
	})?;

	Ok(Some(ItchDatabase {
		games: game_rows
			.filter_map(|row| row.ok_or_log("Failed create itch game from database"))
			.collect(),
		caves: cave_rows
			.filter_map(|row| row.ok_or_log("Failed create itch cave from database"))
			.collect(),
	}))
}
