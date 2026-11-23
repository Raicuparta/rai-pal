use fs_err as fs;
use std::{
	collections::HashSet,
	path::{Path, PathBuf},
	time::{Instant, SystemTime, UNIX_EPOCH},
};

use rusqlite::{OpenFlags, OptionalExtension};
use strum::IntoEnumIterator;

use crate::{
	app_paths,
	debug::LoggableInstant,
	game::Game,
	game_external_id::{ExternalIdProvider, ExternalIds},
	game_providers::game_provider::GameProviderId,
	games_query::{FilterGroup, GamesQuery, GamesSortBy, InstallState},
	local_database::{
		app_database::{AppDatabase, Db},
		mod_database::{self, ModDatabase},
		rusqlite_extensions::{JsonData, RowExt},
	},
	operating_system::OperatingSystem,
	path_extensions::{AsValidStr, PathExt},
	remote_game,
	result::{ContextResult, Error, Result},
};

pub trait GameDatabase {
	fn insert_game(&self, game: &Game);
	fn refresh_games(&self, provider_id: &GameProviderId, games: &[Game], max_time: u64) -> Result;
	fn refresh_game_mod_tags(&self) -> Result;
	fn refresh_game_mod_tags_for_game(&self, provider_id: &GameProviderId, game_id: &str)
	-> Result;
	fn get_game(&self, provider_id: &GameProviderId, game_id: &str) -> Result<Game>;
	fn get_game_ids(&self, query: Option<GamesQuery>) -> Result<GameIdsResponse>;
	fn get_providers_with_games(&self) -> Result<HashSet<GameProviderId>>;
}

pub struct GameId {
	pub provider_id: GameProviderId,
	pub game_id: String,
}

pub struct GameIdsResponse {
	pub game_ids: Vec<GameId>,
	pub total_count: u32,
}

pub(crate) const REMOTE_GAME_JOIN: &str = r"
	LEFT JOIN main.game_remote_metadata rg ON rg.provider_id = g.provider_id AND rg.game_id = g.game_id
";

fn build_nullable_exclusion_filter<T: std::fmt::Display + std::hash::Hash + std::cmp::Eq>(
	column_expr: &str,
	group: &FilterGroup<T>,
	params: &mut Vec<Box<dyn rusqlite::types::ToSql>>,
) -> Option<String> {
	let mut conditions: Vec<String> = Vec::new();

	if group.unknown.as_ref().is_some_and(|item| !item.enabled) {
		conditions.push(format!("{column_expr} IS NOT NULL"));
	}

	let disabled: Vec<&T> = group
		.known
		.iter()
		.filter(|(_, item)| !item.enabled)
		.map(|(val, _)| val)
		.collect();

	if !disabled.is_empty() {
		let placeholders: Vec<_> = disabled
			.iter()
			.map(|val| {
				params.push(Box::new(val.to_string()));
				"?".to_string()
			})
			.collect();
		let cond = if group.unknown.as_ref().is_none_or(|item| item.enabled) {
			format!(
				"{column_expr} NOT IN ({}) OR {column_expr} IS NULL",
				placeholders.join(", ")
			)
		} else {
			format!("{column_expr} NOT IN ({})", placeholders.join(", "))
		};
		conditions.push(cond);
	}

	if conditions.is_empty() {
		None
	} else {
		Some(format!("({})", conditions.join(" AND ")))
	}
}

fn build_supported_os_inclusion_filter(
	group: &FilterGroup<OperatingSystem>,
	params: &mut Vec<Box<dyn rusqlite::types::ToSql>>,
) -> Option<String> {
	let disabled: Vec<&OperatingSystem> = group
		.known
		.iter()
		.filter(|(_, item)| !item.enabled)
		.map(|(val, _)| val)
		.collect();

	// Unlike single-valued filters, a game can support several operating systems,
	// so here the enabled values select which games to keep instead of which to hide.
	let enabled: Vec<OperatingSystem> = OperatingSystem::iter()
		.filter(|os| !disabled.contains(&os))
		.collect();

	let unknown_enabled = group.unknown.as_ref().is_none_or(|item| item.enabled);

	if disabled.is_empty() && unknown_enabled {
		return None;
	}

	if enabled.is_empty() && !unknown_enabled {
		return Some("1=0".to_string());
	}

	let mut branches: Vec<String> = Vec::new();

	if !enabled.is_empty() {
		let like_conditions: Vec<String> = enabled
			.iter()
			.map(|os| {
				params.push(Box::new(format!("%\"{os}\"%")));
				"g.supported_os LIKE ?".to_string()
			})
			.collect();
		branches.push(format!(
			"(g.supported_os IS NOT NULL AND g.supported_os <> '[]' AND ({}))",
			like_conditions.join(" OR ")
		));
	}

	if unknown_enabled {
		branches.push("(g.supported_os IS NULL OR g.supported_os = '[]')".to_string());
	}

	Some(format!("({})", branches.join(" OR ")))
}

const MOD_COMPATIBILITY_FILTER: &str = r"
	(m.engine_key IS NULL OR m.engine_key = COALESCE(ig.engine_brand, rg.engine_brand))
	AND (m.unity_backend_key IS NULL OR ig.unity_backend IS NULL OR m.unity_backend_key = ig.unity_backend)
	AND (m.architecture_key IS NULL OR ig.architecture IS NULL OR m.architecture_key = ig.architecture)
	AND (m.game_os_key IS NULL OR ig.executable_os IS NULL OR m.game_os_key = ig.executable_os)
	AND (m.host_os_key IS NULL OR m.host_os_key = ?)
	AND (
		COALESCE(ig.engine_version_major, rg.engine_version_major) IS NULL
		OR (
			(m.engine_min_major IS NULL OR NOT (
				m.engine_min_major > COALESCE(ig.engine_version_major, rg.engine_version_major)
				OR (
					m.engine_min_major = COALESCE(ig.engine_version_major, rg.engine_version_major)
					AND m.engine_min_minor IS NOT NULL
					AND COALESCE(ig.engine_version_minor, rg.engine_version_minor) IS NOT NULL
					AND (
						m.engine_min_minor > COALESCE(ig.engine_version_minor, rg.engine_version_minor)
						OR (
							m.engine_min_minor = COALESCE(ig.engine_version_minor, rg.engine_version_minor)
							AND m.engine_min_patch IS NOT NULL
							AND COALESCE(ig.engine_version_patch, rg.engine_version_patch) IS NOT NULL
							AND m.engine_min_patch > COALESCE(ig.engine_version_patch, rg.engine_version_patch)
						)
					)
				)
			))
			AND (m.engine_max_major IS NULL OR NOT (
				m.engine_max_major < COALESCE(ig.engine_version_major, rg.engine_version_major)
				OR (
					m.engine_max_major = COALESCE(ig.engine_version_major, rg.engine_version_major)
					AND m.engine_max_minor IS NOT NULL
					AND COALESCE(ig.engine_version_minor, rg.engine_version_minor) IS NOT NULL
					AND (
						m.engine_max_minor < COALESCE(ig.engine_version_minor, rg.engine_version_minor)
						OR (
							m.engine_max_minor = COALESCE(ig.engine_version_minor, rg.engine_version_minor)
							AND m.engine_max_patch IS NOT NULL
							AND COALESCE(ig.engine_version_patch, rg.engine_version_patch) IS NOT NULL
							AND m.engine_max_patch < COALESCE(ig.engine_version_patch, rg.engine_version_patch)
						)
					)
				)
			))
		)
	)
";

fn build_mod_tag_inclusion_filter(
	group: &FilterGroup<String>,
	filterable_tags: &[String],
	params: &mut Vec<Box<dyn rusqlite::types::ToSql>>,
) -> Option<String> {
	if filterable_tags.is_empty() {
		return None;
	}

	let disabled: HashSet<&String> = group
		.known
		.iter()
		.filter(|(tag, item)| !item.enabled && filterable_tags.contains(tag))
		.map(|(tag, _)| tag)
		.collect();
	let unknown_enabled = group.unknown.as_ref().is_none_or(|item| item.enabled);

	if disabled.is_empty() && unknown_enabled {
		return None;
	}

	let enabled: Vec<&String> = filterable_tags
		.iter()
		.filter(|tag| !disabled.contains(tag))
		.collect();

	let mut branches: Vec<String> = Vec::new();

	if !enabled.is_empty() {
		let placeholders: Vec<String> = enabled
			.iter()
			.map(|tag| {
				params.push(Box::new((*tag).clone()));
				"?".to_string()
			})
			.collect();
		branches.push(format!(
			"EXISTS (
				SELECT 1
				FROM main.game_mod_tags game_mod_tag
				WHERE game_mod_tag.provider_id = g.provider_id
					AND game_mod_tag.game_id = g.game_id
					AND game_mod_tag.tag IN ({})
			)",
			placeholders.join(", ")
		));
	}

	if unknown_enabled {
		branches.push(
			"NOT EXISTS (
				SELECT 1
				FROM main.game_mod_tags game_mod_tag
				WHERE game_mod_tag.provider_id = g.provider_id
					AND game_mod_tag.game_id = g.game_id
			)"
			.to_string(),
		);
	}

	if branches.is_empty() {
		Some("1=0".to_string())
	} else {
		Some(format!("({})", branches.join(" OR ")))
	}
}

fn build_game_mod_tags_cte(
	filterable_tags: &[String],
	params: &mut Vec<Box<dyn rusqlite::types::ToSql>>,
	game_filter: Option<(&GameProviderId, &str)>,
) -> String {
	let tags_json = serde_json::to_string(filterable_tags).unwrap_or_else(|_| "[]".to_string());
	params.push(Box::new(tags_json));

	let current_os = OperatingSystem::get_current().to_string();

	for _ in 0..2 {
		params.push(Box::new(current_os.clone()));

		if let Some((provider_id, game_id)) = game_filter {
			params.push(Box::new(provider_id.to_string()));
			params.push(Box::new(game_id.to_string()));
		}
	}

	let game_filter_sql = if game_filter.is_some() {
		"AND g.provider_id = ? AND g.game_id = ?"
	} else {
		""
	};

	format!(
		r"
		WITH filterable_tags AS MATERIALIZED (
			SELECT value AS tag_id FROM json_each(?)
		),
		computed_game_mod_tags AS MATERIALIZED (
			SELECT DISTINCT g.provider_id, g.game_id, mod_tag.tag AS tag
			FROM main.games g
			LEFT JOIN main.installed_games ig ON g.provider_id = ig.provider_id AND g.game_id = ig.game_id
			{REMOTE_GAME_JOIN}
			INNER JOIN main.game_external_ids game_external_id
				ON game_external_id.provider_id = g.provider_id
				AND game_external_id.game_id = g.game_id
			INNER JOIN main.mod_game_ids mod_game_id
				ON mod_game_id.external_id_provider = CASE game_external_id.external_id_provider
					WHEN 'Manual' THEN 'NormalizedTitle'
					ELSE game_external_id.external_id_provider
				END
				AND mod_game_id.external_id = game_external_id.external_id
			INNER JOIN main.mods m ON m.id = mod_game_id.mod_id
			INNER JOIN main.mod_tag_links mod_tag ON mod_tag.mod_id = m.id
			INNER JOIN filterable_tags ON filterable_tags.tag_id = mod_tag.tag
			WHERE COALESCE(m.hide_from_game_mods_list, 0) = 0
				AND {MOD_COMPATIBILITY_FILTER}
				{game_filter_sql}
			UNION
			SELECT DISTINCT g.provider_id, g.game_id, mod_tag.tag AS tag
			FROM main.games g
			LEFT JOIN main.installed_games ig ON g.provider_id = ig.provider_id AND g.game_id = ig.game_id
			{REMOTE_GAME_JOIN}
			INNER JOIN main.mods m ON m.has_game_ids = 0
			INNER JOIN main.mod_tag_links mod_tag ON mod_tag.mod_id = m.id
			INNER JOIN filterable_tags ON filterable_tags.tag_id = mod_tag.tag
			WHERE COALESCE(m.hide_from_game_mods_list, 0) = 0
				AND {MOD_COMPATIBILITY_FILTER}
				{game_filter_sql}
		)
		"
	)
}

fn try_refresh_game_mod_tags(
	connection_mutex: &Db,
	game_filter: Option<(&GameProviderId, &str)>,
) -> Result {
	let filterable_tags = connection_mutex
		.get_mod_tags()?
		.into_iter()
		.filter(|(_, tag)| tag.filterable == Some(true))
		.map(|(tag_id, _)| tag_id)
		.collect::<Vec<_>>();

	let mut params = Vec::<Box<dyn rusqlite::types::ToSql>>::new();
	let cte = build_game_mod_tags_cte(&filterable_tags, &mut params, game_filter);

	let mut connection = connection_mutex.write_db()?;
	let transaction = connection.transaction()?;

	if let Some((provider_id, game_id)) = game_filter {
		transaction
			.prepare_cached("DELETE FROM main.game_mod_tags WHERE provider_id = ? AND game_id = ?")?
			.execute(rusqlite::params![provider_id, game_id])?;
	} else {
		transaction.execute("DELETE FROM main.game_mod_tags", [])?;
	}

	let sql = format!(
		"{cte} INSERT INTO main.game_mod_tags (provider_id, game_id, tag) SELECT provider_id, game_id, tag FROM computed_game_mod_tags"
	);

	let param_refs: Vec<&dyn rusqlite::types::ToSql> =
		params.iter().map(|param| &**param).collect();

	transaction.execute(&sql, param_refs.as_slice())?;
	transaction.commit()?;
	drop(connection);

	Ok(())
}

impl GameDatabase for Db {
	fn insert_game(&self, game: &Game) {
		if let Err(err) = try_insert_games(self, std::slice::from_ref(game)) {
			log::error!(
				"Failed to insert game ({}/{}) into local database: {}",
				game.provider_id,
				game.game_id,
				err
			);
			return;
		}

		if let Err(err) = self.refresh_game_mod_tags_for_game(&game.provider_id, &game.game_id) {
			log::error!(
				"Failed to refresh mod tags for game ({}/{}): {}",
				game.provider_id,
				game.game_id,
				err
			);
		}
	}

	fn refresh_games(&self, provider_id: &GameProviderId, games: &[Game], max_time: u64) -> Result {
		let mut connection = self.write_db()?;
		let transaction = connection.transaction()?;

		insert_games_in_transaction(&transaction, games)?;
		remove_stale_games(&transaction, *provider_id, max_time)?;

		transaction.commit()?;
		drop(connection);

		Ok(())
	}

	fn refresh_game_mod_tags(&self) -> Result {
		try_refresh_game_mod_tags(self, None)
	}

	fn refresh_game_mod_tags_for_game(
		&self,
		provider_id: &GameProviderId,
		game_id: &str,
	) -> Result {
		try_refresh_game_mod_tags(self, Some((provider_id, game_id)))
	}

	fn get_game(&self, provider_id: &GameProviderId, game_id: &str) -> Result<Game> {
		let db = self.read_db()?;

		let mut game = db
			.prepare_cached(&format!(
				r"
		SELECT
			g.provider_id,
			g.game_id,
			g.display_title,
			g.title_discriminator,
			g.thumbnail_url,
			COALESCE(g.release_date, rg.release_date) AS release_date,
			g.tags,
			g.provider_commands,
			ig.exe_path,
			ig.unity_backend,
			ig.architecture,
			ig.executable_os,
			COALESCE(ig.engine_brand, rg.engine_brand) AS engine_brand,
			COALESCE(ig.engine_version_major, rg.engine_version_major) AS engine_version_major,
			COALESCE(ig.engine_version_minor, rg.engine_version_minor) AS engine_version_minor,
			COALESCE(ig.engine_version_patch, rg.engine_version_patch) AS engine_version_patch,
			COALESCE(ig.engine_version_display, rg.engine_version_display) AS engine_version_display,
			g.supported_os
		FROM main.games g
		LEFT JOIN main.installed_games ig ON g.provider_id = ig.provider_id AND g.game_id = ig.game_id
		{REMOTE_GAME_JOIN}
		WHERE g.provider_id = $1 AND g.game_id = $2
		LIMIT 1
	",
			))?
			.query_row([provider_id.to_string(), game_id.to_string()], |row| {
				Ok(Game {
					provider_id: row.get(0)?,
					game_id: row.get(1)?,
					external_ids: ExternalIds::default(),
					display_title: row.get(2)?,
					title_discriminator: row.get(3)?,
					thumbnail_url: row.get(4)?,
					release_date_rfc3339: row.get(5)?,
					tags: row.get_json(6)?,
					provider_commands: row.get_json(7)?,
					exe_path: row.get_path(8)?,
					unity_backend: row.get(9)?,
					architecture: row.get(10)?,
					executable_os: row.get(11)?,
					engine_brand: row.get(12)?,
					engine_version_major: row.get(13)?,
					engine_version_minor: row.get(14)?,
					engine_version_patch: row.get(15)?,
					engine_version_display: row.get(16)?,
					supported_os: row.get_json(17)?,
				})
			})
			.optional()?
			.ok_or_else(|| Error::GameNotFound(format!("{provider_id}/{game_id}")))?;

		let external_ids = db
			.prepare_cached(
				"SELECT external_id_provider, external_id FROM main.game_external_ids WHERE provider_id = $1 AND game_id = $2",
			)?
			.query_map(
				rusqlite::params![game.provider_id, game.game_id],
				|row| {
					Ok((
						row.get::<_, ExternalIdProvider>(0)?,
						row.get::<_, String>(1)?,
					))
				},
			)?
			.collect::<rusqlite::Result<Vec<_>>>()?;

		for (provider, external_id) in external_ids {
			game.external_ids.insert(provider, external_id);
		}

		Ok(game)
	}

	fn get_game_ids(&self, query: Option<GamesQuery>) -> Result<GameIdsResponse> {
		let search = query.as_ref().map_or_default(|q| q.search.clone());

		// Build sorting logic
		let sort_columns = match query.as_ref().map(|q| q.sort_by) {
			Some(GamesSortBy::ReleaseDate) => vec!["COALESCE(g.release_date, rg.release_date)"],
			Some(GamesSortBy::Engine) => vec![
				"COALESCE(ig.engine_brand, rg.engine_brand)",
				"COALESCE(ig.engine_version_major, rg.engine_version_major)",
				"COALESCE(ig.engine_version_minor, rg.engine_version_minor)",
				"COALESCE(ig.engine_version_patch, rg.engine_version_patch)",
			],
			_ => vec!["g.display_title COLLATE NOCASE"],
		};

		let sort_order = if query.as_ref().is_some_and(|q| q.sort_descending) {
			"DESC"
		} else {
			"ASC"
		};

		// Build filtering logic dynamically with parameterized queries
		let mut filters = Vec::<String>::new();
		let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

		if let Some(filter) = query.as_ref().map(|q| &q.filter) {
			// Installed filter
			let installed_disabled = filter
				.installed
				.known
				.get(&InstallState::Installed)
				.is_some_and(|item| !item.enabled);
			let not_installed_disabled = filter
				.installed
				.known
				.get(&InstallState::NotInstalled)
				.is_some_and(|item| !item.enabled);
			if installed_disabled {
				filters.push("ig.exe_path IS NULL".to_string());
			}
			if not_installed_disabled {
				filters.push("ig.exe_path IS NOT NULL".to_string());
			}

			// Providers filter
			{
				let conditions: Vec<String> = filter
					.providers
					.known
					.iter()
					.filter(|(_, item)| !item.enabled)
					.map(|(provider, _)| {
						params.push(Box::new(provider.to_string()));
						"g.provider_id = ?".to_string()
					})
					.collect();
				if !conditions.is_empty() {
					filters.push(format!("NOT ({})", conditions.join(" OR ")));
				}
			}

			// Tags filter
			{
				let mut conditions: Vec<String> = Vec::new();
				if filter
					.tags
					.unknown
					.as_ref()
					.is_some_and(|item| !item.enabled)
				{
					conditions.push("g.tags <> '[]'".to_string());
				}
				for (tag, item) in &filter.tags.known {
					if !item.enabled {
						params.push(Box::new(format!(r#"%"{tag}"%"#)));
						conditions.push("g.tags NOT LIKE ?".to_string());
					}
				}
				if !conditions.is_empty() {
					filters.push(format!("({})", conditions.join(" AND ")));
				}
			}

			// Engines filter
			if let Some(cond) = build_nullable_exclusion_filter(
				"COALESCE(ig.engine_brand, rg.engine_brand)",
				&filter.engines,
				&mut params,
			) {
				filters.push(cond);
			}

			// Unity backends filter
			if let Some(cond) = build_nullable_exclusion_filter(
				"ig.unity_backend",
				&filter.unity_backends,
				&mut params,
			) {
				filters.push(cond);
			}

			// Architectures filter
			if let Some(cond) = build_nullable_exclusion_filter(
				"ig.architecture",
				&filter.architectures,
				&mut params,
			) {
				filters.push(cond);
			}

			// Executable operating system filter
			if let Some(cond) = build_nullable_exclusion_filter(
				"ig.executable_os",
				&filter.executable_os,
				&mut params,
			) {
				filters.push(cond);
			}

			// Supported operating systems filter
			if let Some(cond) =
				build_supported_os_inclusion_filter(&filter.supported_os, &mut params)
			{
				filters.push(cond);
			}

			// Mod tags filter
			// The "unknown" value means the game has no compatible mod from any
			// filterable tag. This lets "Only" on one tag actually show only
			// games that have a mod from that tag.
			let filterable_mod_tags: Vec<String> = self
				.get_mod_tags()?
				.into_iter()
				.filter(|(_, tag)| tag.filterable == Some(true))
				.map(|(tag_id, _)| tag_id)
				.collect();

			if let Some(cond) =
				build_mod_tag_inclusion_filter(&filter.mod_tags, &filterable_mod_tags, &mut params)
			{
				filters.push(cond);
			}
		}

		let trimmed_search = search.trim();
		if !trimmed_search.is_empty() {
			params.push(Box::new(format!("%{trimmed_search}%")));
			params.push(Box::new(format!("%{trimmed_search}%")));
			filters.push(
				"(g.display_title LIKE ? OR EXISTS (SELECT 1 FROM main.game_external_ids game_external_id WHERE game_external_id.provider_id = g.provider_id AND game_external_id.game_id = g.game_id AND game_external_id.external_id_provider IN ('NormalizedTitle', 'Manual') AND game_external_id.external_id LIKE ?))"
					.to_string(),
			);
		}

		let where_clause = if filters.is_empty() {
			"1=1".to_string()
		} else {
			filters.join(" AND ")
		};

		let sort_clause = sort_columns
			.iter()
			.map(|col| format!("{col} {sort_order}"))
			.collect::<Vec<_>>()
			.join(", ");

		let sql_query = format!(
			r"
			SELECT DISTINCT
					g.provider_id as provider_id,
					g.game_id as game_id
			FROM main.games g
			LEFT JOIN main.installed_games ig ON g.provider_id = ig.provider_id AND g.game_id = ig.game_id
			{REMOTE_GAME_JOIN}
			WHERE {where_clause}
			ORDER BY {sort_clause}
			",
		);

		let param_refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|p| &**p).collect();
		let game_ids = self
			.read_db()?
			.prepare(&sql_query)?
			.query_map(param_refs.as_slice(), |row| {
				Ok(GameId {
					provider_id: row.get(0)?,
					game_id: row.get(1)?,
				})
			})?
			.filter_map(|game_id| match game_id {
				Ok(id) => Some(id),
				Err(err) => {
					log::warn!("Failed to read game from local database: {err}");
					None
				}
			})
			.collect();

		let total_count = self
			.read_db()?
			.prepare_cached(
				r"
			SELECT COUNT(*)
			FROM main.games g
		",
			)?
			.query_row([], |row| row.get::<_, i64>(0))?;

		Ok(GameIdsResponse {
			game_ids,
			total_count: u32::try_from(total_count)?,
		})
	}

	fn get_providers_with_games(&self) -> Result<HashSet<GameProviderId>> {
		let mut providers = HashSet::new();
		{
			let db = self.read_db()?;
			let mut statement =
				db.prepare_cached("SELECT DISTINCT provider_id FROM main.games;")?;
			let rows = statement.query_map([], |row| row.get::<_, GameProviderId>(0))?;

			for provider in rows {
				match provider {
					Ok(provider) => {
						providers.insert(provider);
					}
					Err(err) => {
						log::warn!("Failed to read provider with games from local database: {err}");
					}
				}
			}

			drop(statement);
			drop(db);
		}

		Ok(providers)
	}
}

fn remove_stale_games(
	transaction: &rusqlite::Transaction,
	provider_id: GameProviderId,
	max_time: u64,
) -> Result {
	transaction.prepare_cached(
		"DELETE FROM main.installed_games WHERE provider_id = $1 AND game_id IN (SELECT game_id FROM main.games WHERE provider_id = $1 AND created_at < $2)",
	)?
	.execute(rusqlite::params![provider_id, max_time.cast_signed()])?;
	transaction.prepare_cached(
		"DELETE FROM main.game_external_ids WHERE provider_id = $1 AND game_id IN (SELECT game_id FROM main.games WHERE provider_id = $1 AND created_at < $2)",
	)?
	.execute(rusqlite::params![provider_id, max_time.cast_signed()])?;
	transaction
		.prepare_cached("DELETE FROM main.games WHERE provider_id = $1 AND created_at < $2")?
		.execute(rusqlite::params![provider_id, max_time.cast_signed()])?;

	Ok(())
}

fn try_insert_games(connection_mutex: &Db, games: &[Game]) -> Result {
	let mut connection = connection_mutex.write_db()?;
	let transaction = connection.transaction()?;

	insert_games_in_transaction(&transaction, games)?;

	transaction.commit()?;
	drop(connection);

	Ok(())
}

fn insert_games_in_transaction(transaction: &rusqlite::Transaction, games: &[Game]) -> Result {
	let created_at = SystemTime::now()
		.duration_since(UNIX_EPOCH)?
		.as_secs()
		.cast_signed();

	for game in games {
		insert_game_in_transaction(transaction, game, created_at)?;
	}

	Ok(())
}

fn insert_game_in_transaction(
	transaction: &rusqlite::Transaction,
	game: &Game,
	created_at: i64,
) -> Result {
	transaction
		.prepare_cached(
			"INSERT OR REPLACE INTO games (
				provider_id,
				game_id,
				display_title,
				thumbnail_url,
				release_date,
				tags,
				title_discriminator,
				provider_commands,
				exe_path_hash,
				created_at,
				supported_os
			) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)",
		)?
		.execute(rusqlite::params![
			game.provider_id,
			game.game_id.clone(),
			game.display_title.clone(),
			game.thumbnail_url.clone(),
			game.release_date_rfc3339,
			JsonData(game.tags.clone()),
			game.title_discriminator.clone(),
			JsonData(game.provider_commands.clone()),
			game.exe_path
				.as_ref()
				.map(|exe_path| exe_path.as_path().hash_string()),
			created_at,
			JsonData(game.supported_os.clone()),
		])?;

	transaction
		.prepare_cached(
			"DELETE FROM main.game_external_ids WHERE provider_id = $1 AND game_id = $2",
		)?
		.execute(rusqlite::params![game.provider_id, game.game_id])?;

	if let Some(exe_path) = game.exe_path.as_ref() {
		transaction
			.prepare_cached(
				"INSERT OR REPLACE INTO installed_games (
					provider_id,
					game_id,
					exe_path,
					engine_brand,
					engine_version_major,
					engine_version_minor,
					engine_version_patch,
					engine_version_display,
					unity_backend,
					architecture,
					executable_os
				)
				 VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)",
			)?
			.execute(rusqlite::params![
				game.provider_id,
				game.game_id.clone(),
				exe_path.try_to_str()?,
				game.engine_brand,
				game.engine_version_major,
				game.engine_version_minor,
				game.engine_version_patch,
				game.engine_version_display.clone(),
				game.unity_backend.clone(),
				game.architecture.clone(),
				game.executable_os,
			])?;
	}

	for (provider, external_ids) in &game.external_ids.map {
		let mut statement = transaction.prepare_cached(
			"INSERT OR REPLACE INTO main.game_external_ids (provider_id, game_id, external_id_provider, external_id) VALUES ($1, $2, $3, $4)",
		)?;

		for external_id in external_ids {
			statement.execute(rusqlite::params![
				game.provider_id,
				game.game_id,
				provider,
				external_id,
			])?;
		}
	}

	Ok(())
}

pub fn try_create() -> Result<Db> {
	match create() {
		Ok(db) => Ok(db),
		Err(initial_error) => {
			log::error!(
				"Failed to set up local databases. Deleting them and retrying once. Error: {initial_error}"
			);
			cleanup_database_files();

			match create() {
				Ok(db) => Ok(db),
				Err(retry_error) => {
					log::error!(
						"Failed to set up local databases after cleanup retry. Giving up. Error: {retry_error}"
					);
					Err(retry_error)
				}
			}
		}
	}
}

pub fn recreate_if_missing(database: &Db) -> Result {
	let path = db_file_path()?;
	if path.is_file() {
		return Ok(());
	}

	log::warn!(
		"Local database file is missing, recreating it: {}",
		path.display()
	);

	database.replace_connection(create_connection(&path)?)?;

	Ok(())
}

pub fn attach_remote(database: &Db, path: &Path) -> Result {
	let mut connection = database.write_db()?;
	attach_remote_to_connection(&mut connection, path)
}

fn attach_remote_to_connection(connection: &mut rusqlite::Connection, path: &Path) -> Result {
	let mut instant = Instant::now();
	instant.log_next("Attaching remote database...");

	if !path.is_file() {
		return Ok(());
	}

	if is_database_attached(connection, "remote_db")? {
		connection.execute("DETACH DATABASE remote_db;", [])?;
	}

	let attach_work_result = (|| -> Result {
		connection.execute("ATTACH DATABASE ?1 AS remote_db;", [path.try_to_str()?])?;

		let transaction = connection.transaction()?;

		transaction.execute("DELETE FROM main.game_remote_metadata", [])?;

		transaction.execute(
			r"
			INSERT OR REPLACE INTO main.game_remote_metadata (
				provider_id,
				game_id,
				engine_brand,
				engine_version_major,
				engine_version_minor,
				engine_version_patch,
				engine_version_display,
				release_date
			)
			SELECT
				provider_id,
				game_id,
				engine_brand,
				engine_version_major,
				engine_version_minor,
				engine_version_patch,
				engine_version_display,
				release_date
			FROM (
				SELECT
					ge.provider_id AS provider_id,
					ge.game_id AS game_id,
					e.name AS engine_brand,
					rg.engine_version_major AS engine_version_major,
					rg.engine_version_minor AS engine_version_minor,
					rg.engine_version_patch AS engine_version_patch,
					CASE
						WHEN rg.engine_version_major IS NULL THEN NULL
						WHEN rg.engine_version_minor IS NULL THEN CAST(rg.engine_version_major AS TEXT)
						WHEN rg.engine_version_patch IS NULL THEN CAST(rg.engine_version_major AS TEXT) || '.' || CAST(rg.engine_version_minor AS TEXT)
						ELSE CAST(rg.engine_version_major AS TEXT) || '.' || CAST(rg.engine_version_minor AS TEXT) || '.' || CAST(rg.engine_version_patch AS TEXT)
					END AS engine_version_display,
					CASE
						WHEN rg.release_date IS NULL THEN NULL
						ELSE strftime('%Y-%m-%dT%H:%M:%SZ', rg.release_date * 86400, 'unixepoch')
					END AS release_date,
					ROW_NUMBER() OVER (
						PARTITION BY ge.provider_id, ge.game_id
						ORDER BY
							CASE WHEN remote_provider.name = ge.external_id_provider THEN 0 ELSE 1 END,
							rg.engine_brand IS NULL,
							rg.release_date IS NULL,
							rg.release_date DESC,
							rg.provider_id
					) AS row_number
				FROM main.game_external_ids ge
				INNER JOIN remote_db.providers remote_provider
					ON remote_provider.name = CASE ge.external_id_provider
						WHEN 'Manual' THEN 'NormalizedTitle'
						ELSE ge.external_id_provider
					END
				INNER JOIN remote_db.games rg
					ON rg.provider_id = remote_provider.id
					AND rg.external_id = ge.external_id
				LEFT JOIN remote_db.engines e ON e.id = rg.engine_brand
			)
			WHERE row_number = 1
			",
			[],
		)?;
		transaction.commit()?;
		Ok(())
	})();

	let detach_result = match is_database_attached(connection, "remote_db") {
		Ok(true) => connection
			.execute("DETACH DATABASE remote_db;", [])
			.map(|_| ())
			.map_err(Into::into),
		Ok(false) => Ok(()),
		Err(err) => Err(err),
	};

	match (attach_work_result, detach_result) {
		(Ok(()), Ok(())) => {}
		(Err(work_err), Ok(())) => return Err(work_err),
		(Ok(()), Err(detach_err)) => return Err(detach_err),
		(Err(work_err), Err(detach_err)) => {
			log::warn!("Failed to detach remote_db after attach_remote error: {detach_err}");
			return Err(work_err);
		}
	}

	instant.log_next("Finished attaching up remote games database.");

	Ok(())
}

fn cleanup_database_files() {
	cleanup_database_file(db_file_path(), "local");
	cleanup_database_file(remote_game::get_database_file_path(), "remote");
}

fn cleanup_database_file(path_result: Result<PathBuf>, database_name: &str) {
	let path = match path_result {
		Ok(path) => path,
		Err(path_error) => {
			log::warn!("Failed to resolve {database_name} database path for cleanup: {path_error}");
			return;
		}
	};

	match fs::remove_file(&path) {
		Ok(()) => {}
		Err(remove_error) if remove_error.kind() == std::io::ErrorKind::NotFound => {}
		Err(remove_error) => {
			log::warn!(
				"Failed to delete {database_name} database after create failure ({}): {remove_error}",
				path.display()
			);
		}
	}
}

fn create() -> Result<Db> {
	let mut instant = Instant::now();
	instant.log_next("Creating local database...");

	let path = db_file_path()?;
	let database = Db::new(path.clone(), create_connection(&path)?);

	instant.log_next("Created local database!");

	Ok(database)
}

fn create_connection(path: &Path) -> Result<rusqlite::Connection> {
	let mut connection = open_database_connection(path)?;
	create_tables(&connection)?;
	mod_database::create_tables(&connection)?;
	attach_remote_to_connection(&mut connection, &remote_game::get_database_file_path()?)?;
	Ok(connection)
}

fn create_tables(connection: &rusqlite::Connection) -> Result {
	connection.execute_batch(
		r"
		CREATE TABLE IF NOT EXISTS games (
			provider_id TEXT NOT NULL,
			game_id TEXT NOT NULL,
			display_title TEXT NOT NULL,
			title_discriminator TEXT,
			thumbnail_url TEXT,
			tags TEXT,
			release_date INTEGER,
			provider_commands TEXT,
			exe_path_hash TEXT,
			created_at INTEGER,
			supported_os TEXT,
			PRIMARY KEY (provider_id, game_id)
		);

		CREATE INDEX IF NOT EXISTS idx_games_exe_path_hash ON games(exe_path_hash);

		CREATE TABLE IF NOT EXISTS game_external_ids (
			provider_id TEXT NOT NULL,
			game_id TEXT NOT NULL,
			external_id_provider TEXT NOT NULL,
			external_id TEXT NOT NULL,
			FOREIGN KEY (provider_id, game_id) REFERENCES games(provider_id, game_id) ON DELETE CASCADE,
			PRIMARY KEY (provider_id, game_id, external_id_provider, external_id)
		);

		CREATE INDEX IF NOT EXISTS idx_game_external_ids_lookup ON game_external_ids(external_id_provider, external_id);

		CREATE TABLE IF NOT EXISTS game_mod_tags (
			provider_id TEXT NOT NULL,
			game_id TEXT NOT NULL,
			tag TEXT NOT NULL,
			PRIMARY KEY (provider_id, game_id, tag)
		) WITHOUT ROWID;

		CREATE TABLE IF NOT EXISTS installed_games (
			provider_id TEXT NOT NULL,
			game_id TEXT NOT NULL,
			exe_path TEXT NOT NULL,
			engine_brand TEXT,
			engine_version_major INTEGER,
			engine_version_minor INTEGER,
			engine_version_patch INTEGER,
			engine_version_display TEXT,
			unity_backend TEXT,
			architecture TEXT,
			executable_os TEXT,
			FOREIGN KEY(provider_id, game_id) REFERENCES games(provider_id, game_id) ON DELETE CASCADE,
			PRIMARY KEY (provider_id, game_id)
		);

		CREATE TABLE IF NOT EXISTS game_remote_metadata (
			provider_id TEXT NOT NULL,
			game_id TEXT NOT NULL,
			engine_brand TEXT,
			engine_version_major INTEGER,
			engine_version_minor INTEGER,
			engine_version_patch INTEGER,
			engine_version_display TEXT,
			release_date TEXT,
			PRIMARY KEY (provider_id, game_id)
		) WITHOUT ROWID;
	",
	)?;

	Ok(())
}

fn open_database_connection(path: &Path) -> Result<rusqlite::Connection> {
	if let Some(parent) = path.parent() {
		fs_err::create_dir_all(parent)?;
	}

	let connection = rusqlite::Connection::open_with_flags(
		path,
		OpenFlags::SQLITE_OPEN_CREATE
			| OpenFlags::SQLITE_OPEN_READ_WRITE
			| OpenFlags::SQLITE_OPEN_PRIVATE_CACHE,
	)
	.with_context(|| format!("failed to open local database `{}`", path.display()))?;

	connection.execute_batch(
		r"
		PRAGMA journal_mode = WAL;
		PRAGMA synchronous = OFF;
		PRAGMA busy_timeout = 5000;
	",
	)?;

	Ok(connection)
}

pub(crate) fn open_reader_connection(path: &Path) -> Result<rusqlite::Connection> {
	let connection = rusqlite::Connection::open_with_flags(
		path,
		OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_PRIVATE_CACHE,
	)
	.with_context(|| format!("failed to open local database `{}`", path.display()))?;

	connection.execute_batch(
		r"
		PRAGMA query_only = ON;
		PRAGMA busy_timeout = 5000;
	",
	)?;

	Ok(connection)
}

fn is_database_attached(connection: &rusqlite::Connection, database_name: &str) -> Result<bool> {
	let databases = connection
		.prepare_cached("PRAGMA database_list;")?
		.query_map([], |row| row.get::<_, String>(1))?
		.collect::<rusqlite::Result<Vec<_>>>()?;

	Ok(databases.iter().any(|name| name == database_name))
}

fn db_file_path() -> Result<PathBuf> {
	app_paths::database_path("local")
}
