use std::{
	sync::{Arc, LazyLock},
	thread,
	time::Duration,
};

use rai_pal_core::{
	game_providers::game_provider::GameProviderId,
	local_database::{app_database::DbMutex, game_database::GameDatabase},
};

use slint::Model;

use crate::{
	MainWindow,
	model::{format_release_date, get_engine_details, tags},
	scheduler::{FetchRequest, FetchScheduler},
};

const WORKER_COUNT: usize = 4;

static HTTP_CLIENT: LazyLock<reqwest::blocking::Client> = LazyLock::new(|| {
	reqwest::blocking::Client::builder()
		.connect_timeout(Duration::from_secs(10))
		.timeout(Duration::from_secs(30))
		.build()
		.unwrap_or_default()
});

struct FetchResult {
	index: usize,
	generation: u64,
	title: String,
	installed: bool,
	tags: Vec<slint::SharedString>,
	engine_brand: String,
	engine_version: String,
	engine_details: String,
	release_day: String,
	release_month: String,
	release_year: String,
	game_json: String,
	thumbnail: Option<Thumbnail>,
}

struct Thumbnail {
	width: u32,
	height: u32,
	pixels: Vec<u8>,
}

fn fetch_thumbnail(url: &str) -> Option<Thumbnail> {
	let response = HTTP_CLIENT.get(url).send().ok()?.error_for_status().ok()?;
	let bytes = response.bytes().ok()?;

	if bytes.starts_with(b"GIF8") {
		return None;
	}

	let image = image::load_from_memory(&bytes).ok()?;
	let image = image.thumbnail(200, 120).to_rgba8();

	Some(Thumbnail {
		width: image.width(),
		height: image.height(),
		pixels: image.into_raw(),
	})
}

fn fetch_game(database: &DbMutex, request: &FetchRequest) -> FetchResult {
	let provider_id = match request.provider_id.parse::<GameProviderId>() {
		Ok(provider_id) => provider_id,
		Err(err) => {
			eprintln!("Invalid provider id {}: {err}", request.provider_id);
			return FetchResult {
				index: request.index,
				generation: request.generation,
				title: format!("{}: {}", request.provider_id, request.game_id),
				installed: false,
				tags: Vec::new(),
				engine_brand: String::new(),
				engine_version: String::new(),
				engine_details: String::new(),
				release_day: String::new(),
				release_month: String::new(),
				release_year: String::new(),
				game_json: String::new(),
				thumbnail: None,
			};
		}
	};

	match database.get_game(&provider_id, &request.game_id) {
		Ok(game) => {
			let (release_day, release_month, release_year) =
				format_release_date(game.release_date_rfc3339.as_deref());
			let engine_brand = game
				.engine_brand
				.map(|brand| brand.to_string())
				.unwrap_or_default();
			let engine_version = game.engine_version_display.clone().unwrap_or_default();
			let engine_details = get_engine_details(&game);
			let game_json = serde_json::to_string_pretty(&game).unwrap_or_default();
			let thumbnail = game.thumbnail_url.as_deref().and_then(fetch_thumbnail);

			FetchResult {
				index: request.index,
				generation: request.generation,
				title: game.full_title(),
				installed: game.exe_path.is_some(),
				tags: tags(&game),
				engine_brand,
				engine_version,
				engine_details,
				release_day,
				release_month,
				release_year,
				game_json,
				thumbnail,
			}
		}
		Err(err) => {
			eprintln!(
				"Failed to load game {}:{}: {err}",
				request.provider_id, request.game_id
			);
			FetchResult {
				index: request.index,
				generation: request.generation,
				title: format!("{}: {}", request.provider_id, request.game_id),
				installed: false,
				tags: Vec::new(),
				engine_brand: String::new(),
				engine_version: String::new(),
				engine_details: String::new(),
				release_day: String::new(),
				release_month: String::new(),
				release_year: String::new(),
				game_json: String::new(),
				thumbnail: None,
			}
		}
	}
}

fn spawn_worker(
	scheduler: Arc<FetchScheduler>,
	database: Arc<DbMutex>,
	window: slint::Weak<MainWindow>,
) {
	thread::spawn(move || {
		while let Some(request) = scheduler.take_next() {
			let result = fetch_game(&database, &request);
			let window = window.clone();
			let ui_scheduler = scheduler.clone();
			let index = result.index;

			let sent = slint::invoke_from_event_loop(move || {
				if result.generation != ui_scheduler.generation() {
					return;
				}

				ui_scheduler.finish(index);

				if !ui_scheduler.should_apply(index) {
					return;
				}

				let Some(window) = window.upgrade() else {
					return;
				};

				let rows = window.get_game_rows();
				let Some(mut row) = rows.row_data(result.index) else {
					return;
				};

				let thumbnail = result.thumbnail.map(|thumbnail| {
					let buffer = slint::SharedPixelBuffer::<slint::Rgba8Pixel>::clone_from_slice(
						&thumbnail.pixels,
						thumbnail.width,
						thumbnail.height,
					);
					slint::Image::from_rgba8(buffer)
				});

				row.title = result.title.into();
				row.installed = result.installed;
				row.tags = std::rc::Rc::new(slint::VecModel::from(result.tags)).into();
				row.engine_brand = result.engine_brand.into();
				row.engine_version = result.engine_version.into();
				row.engine_details = result.engine_details.into();
				row.release_day = result.release_day.into();
				row.release_month = result.release_month.into();
				row.release_year = result.release_year.into();
				row.game_json = result.game_json.into();
				row.has_thumbnail = thumbnail.is_some();
				row.thumbnail = thumbnail.unwrap_or_default();
				row.loaded = true;

				rows.set_row_data(result.index, row);
				ui_scheduler.mark_loaded(result.index);
			});

			if sent.is_err() {
				scheduler.finish(index);
			}
		}
	});
}

pub fn spawn_fetchers(
	database: Arc<DbMutex>,
	window: slint::Weak<MainWindow>,
) -> Arc<FetchScheduler> {
	let scheduler = Arc::new(FetchScheduler::default());

	for _ in 0..WORKER_COUNT {
		spawn_worker(scheduler.clone(), database.clone(), window.clone());
	}

	scheduler
}
