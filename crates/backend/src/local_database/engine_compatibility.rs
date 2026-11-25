#[derive(Clone, Copy)]
pub struct VersionColumns<'a> {
	pub major: &'a str,
	pub minor: &'a str,
	pub patch: &'a str,
}

/// SQL expression that is true when the game's engine version is within the
/// mod's supported range. A game with an unknown engine version is considered
/// compatible with any range.
pub fn engine_version_in_range(
	game: VersionColumns,
	minimum: VersionColumns,
	maximum: VersionColumns,
) -> String {
	let below_minimum = out_of_range(minimum, game, ">");
	let above_maximum = out_of_range(maximum, game, "<");

	format!(
		"({game_major} IS NULL OR (({min_major} IS NULL OR NOT {below_minimum}) AND ({max_major} IS NULL OR NOT {above_maximum})))",
		game_major = game.major,
		min_major = minimum.major,
		max_major = maximum.major,
	)
}

fn out_of_range(bound: VersionColumns, game: VersionColumns, operator: &str) -> String {
	let VersionColumns {
		major: bound_major,
		minor: bound_minor,
		patch: bound_patch,
	} = bound;
	let VersionColumns {
		major: game_major,
		minor: game_minor,
		patch: game_patch,
	} = game;

	format!(
		r"(
	{bound_major} {operator} {game_major}
	OR (
		{bound_major} = {game_major}
		AND {bound_minor} IS NOT NULL
		AND {game_minor} IS NOT NULL
		AND (
			{bound_minor} {operator} {game_minor}
			OR (
				{bound_minor} = {game_minor}
				AND {bound_patch} IS NOT NULL
				AND {game_patch} IS NOT NULL
				AND {bound_patch} {operator} {game_patch}
			)
		)
	)
)"
	)
}
