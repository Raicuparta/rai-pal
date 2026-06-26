use std::{
	collections::HashMap,
	sync::{
		LazyLock,
		Mutex,
	},
	time::{
		Duration,
		Instant,
	},
};

pub trait LoggableInstant {
	fn log_next(&mut self, message: &str);
}

impl LoggableInstant for Instant {
	fn log_next(&mut self, message: &str) {
		let millis = self.elapsed().as_millis();
		println!("{millis:10}ms # {message}");
		*self = Self::now();
	}
}

/// Collects per-engine, per-game processing times for the `process_game` pipeline.
/// Data is accumulated globally and a sorted report can be printed on demand.
#[derive(Default)]
pub struct GameProcessTimings {
	pub records: Vec<(String, String, Duration)>,
}

impl GameProcessTimings {
	pub fn record(&mut self, engine: &str, game_title: &str, duration: Duration) {
		self.records
			.push((engine.to_string(), game_title.to_string(), duration));
	}
}

static GLOBAL_TIMINGS: LazyLock<Mutex<GameProcessTimings>> =
	LazyLock::new(|| Mutex::new(GameProcessTimings::default()));

/// Records a single engine-processing measurement into the global collector.
pub fn record_game_process_time(engine: &str, game_title: &str, duration: Duration) {
	if let Ok(mut timings) = GLOBAL_TIMINGS.lock() {
		timings.record(engine, game_title, duration);
	}
}

/// Prints a comprehensive sorted timing report to stdout.
///
/// The report shows:
/// 1. Total time per engine (sorted by total, descending)
/// 2. Top N slowest games per engine
/// 3. Slowest individual game(s) across all engines
pub fn print_game_process_timing_report() {
	let Ok(timings) = GLOBAL_TIMINGS.lock() else {
		eprintln!("[timing] Could not acquire lock for timing report.");
		return;
	};

	if timings.records.is_empty() {
		println!("\n  === Game Processing Timing Report ===\n  No games were processed.\n");
		return;
	}

	// Group by engine (use owned String keys to avoid borrow issues).
	#[derive(Default)]
	struct EngineEntry {
		total: Duration,
		games: Vec<(String, Duration)>,
	}

	let mut engine_map: HashMap<String, EngineEntry> = HashMap::new();
	for (engine, game, duration) in &timings.records {
		let entry = engine_map.entry(engine.clone()).or_default();
		entry.total += *duration;
		entry.games.push((game.clone(), *duration));
	}

	// Collect engine names sorted by total time descending.
	let mut engine_keys: Vec<&str> = engine_map.keys().map(|s| s.as_str()).collect();
	engine_keys.sort_by(|a, b| {
		let total_a = engine_map
			.get(*a)
			.map(|e| e.total)
			.unwrap_or(Duration::ZERO);
		let total_b = engine_map
			.get(*b)
			.map(|e| e.total)
			.unwrap_or(Duration::ZERO);
		total_b.cmp(&total_a)
	});

	println!();
	println!("  ==================== Game Processing Timing Report ====================");
	println!();

	// --- Section 1: Engine aggregate times ---
	println!("  --- By Engine (total time, descending) ---");
	for engine in &engine_keys {
		if let Some(entry) = engine_map.get(*engine) {
			let count = entry.games.len();
			let avg = if count > 0 {
				entry.total / count as u32
			} else {
				Duration::ZERO
			};
			let total_ms = entry.total.as_millis();
			let avg_ms = avg.as_millis();
			println!(
				"    {engine:8}  {total_ms:>8} ms  total  ({count:3} games,  avg {avg_ms:>5} ms)"
			);
		}
	}
	println!();

	// --- Section 2: Top 10 slowest overall ---
	let mut all_games: Vec<(&String, &String, Duration)> =
		timings.records.iter().map(|(e, g, d)| (e, g, *d)).collect();
	all_games.sort_by(|a, b| b.2.cmp(&a.2));

	println!("  --- Top 10 Slowest Games (all engines) ---");
	for (i, (engine, game, duration)) in all_games.iter().take(10).enumerate() {
		let ms = duration.as_millis();
		println!("    {:<2}. {engine:8}  \"{game}\"  {ms:>8} ms", i + 1);
	}
	println!();

	// --- Section 3: Per-engine top 5 ---
	println!("  --- Top 5 Slowest Games per Engine ---");
	for engine in &engine_keys {
		if let Some(entry) = engine_map.get(*engine) {
			let mut sorted_games = entry.games.clone();
			sorted_games.sort_by(|a, b| b.1.cmp(&a.1));

			println!("    {engine}:");
			for (i, (game, duration)) in sorted_games.iter().take(5).enumerate() {
				let ms = duration.as_millis();
				println!("      {:<2}. \"{game}\"  {ms:>8} ms", i + 1);
			}
			if sorted_games.len() > 5 {
				println!("        ... and {} more", sorted_games.len() - 5);
			}
		}
	}
	println!();
	println!("  ==========================================================================");
	println!();
}

/// Reset all collected timing data (useful between test runs or manual invocations).
pub fn reset_game_process_timings() {
	if let Ok(mut timings) = GLOBAL_TIMINGS.lock() {
		timings.records.clear();
	}
}
