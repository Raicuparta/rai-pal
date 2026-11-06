use std::{
	io::Read,
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
	scheduler::{FetchRequest, FetchScheduler, ThumbnailOutcome},
};

const WORKER_COUNT: usize = 4;
const IMAGE_WORKER_COUNT: usize = 4;

static HTTP_CLIENT: LazyLock<reqwest::blocking::Client> = LazyLock::new(|| {
	reqwest::blocking::Client::builder()
		.connect_timeout(Duration::from_secs(10))
		.timeout(Duration::from_secs(30))
		.build()
		.unwrap_or_default()
});

pub(crate) fn http_client() -> &'static reqwest::blocking::Client {
	&HTTP_CLIENT
}

pub(crate) struct FetchedImage {
	pub width: u32,
	pub height: u32,
	pub pixels: Vec<u8>,
}

impl FetchedImage {
	pub fn into_slint_image(self) -> slint::Image {
		let buffer = slint::SharedPixelBuffer::<slint::Rgba8Pixel>::clone_from_slice(
			&self.pixels,
			self.width,
			self.height,
		);

		slint::Image::from_rgba8(buffer)
	}
}

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
	thumbnail_url: Option<String>,
}

pub(crate) enum ImageFetchOutcome {
	Fetched(FetchedImage),
	Cancelled,
	Failed,
}

pub(crate) fn fetch_image(url: &str, max_width: u32, max_height: u32) -> Option<FetchedImage> {
	match fetch_image_cancellable(url, max_width, max_height, || true) {
		ImageFetchOutcome::Fetched(image) => Some(image),
		ImageFetchOutcome::Cancelled | ImageFetchOutcome::Failed => None,
	}
}

pub(crate) fn fetch_image_cancellable<F>(
	url: &str,
	max_width: u32,
	max_height: u32,
	mut should_continue: F,
) -> ImageFetchOutcome
where
	F: FnMut() -> bool,
{
	let response = match HTTP_CLIENT.get(url).send() {
		Ok(response) => response,
		Err(_) => return ImageFetchOutcome::Failed,
	};
	let mut response = match response.error_for_status() {
		Ok(response) => response,
		Err(_) => return ImageFetchOutcome::Failed,
	};

	let mut bytes = Vec::new();
	let mut chunk = [0u8; 16 * 1024];

	loop {
		if !should_continue() {
			return ImageFetchOutcome::Cancelled;
		}

		match response.read(&mut chunk) {
			Ok(0) => break,
			Ok(length) => bytes.extend_from_slice(&chunk[..length]),
			Err(_) => return ImageFetchOutcome::Failed,
		}
	}

	if !should_continue() {
		return ImageFetchOutcome::Cancelled;
	}

	if bytes.starts_with(b"GIF8") {
		return ImageFetchOutcome::Failed;
	}

	let Ok(image) = image::load_from_memory(&bytes) else {
		return ImageFetchOutcome::Failed;
	};
	let image = image.thumbnail(max_width, max_height).to_rgba8();

	ImageFetchOutcome::Fetched(FetchedImage {
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
				thumbnail_url: None,
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
			let thumbnail_url = game.thumbnail_url.clone();

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
				thumbnail_url,
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
				thumbnail_url: None,
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

				let thumbnail_url = result.thumbnail_url;

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
				row.loaded = true;

				rows.set_row_data(result.index, row);
				ui_scheduler.mark_loaded(result.index);

				if let Some(url) = thumbnail_url {
					ui_scheduler.enqueue_thumbnail(result.index, result.generation, url);
				}
			});

			if sent.is_err() {
				scheduler.finish(index);
			}
		}
	});
}

fn spawn_image_worker(scheduler: Arc<FetchScheduler>, window: slint::Weak<MainWindow>) {
	thread::spawn(move || {
		while let Some(job) = scheduler.take_next_thumbnail() {
			let cancel_scheduler = scheduler.clone();
			let cancel_job = job.clone();
			let outcome = fetch_image_cancellable(&job.url, 200, 120, move || {
				cancel_scheduler.thumbnail_wanted(&cancel_job)
			});

			match outcome {
				ImageFetchOutcome::Fetched(image) => {
					let window = window.clone();
					let ui_scheduler = scheduler.clone();
					let ui_job = job.clone();

					let sent = slint::invoke_from_event_loop(move || {
						let indices = ui_scheduler.finish_thumbnail(
							&ui_job,
							ThumbnailOutcome::Success,
						);

						if ui_job.generation != ui_scheduler.generation() || indices.is_empty() {
							return;
						}

						let Some(window) = window.upgrade() else {
							return;
						};

						let rows = window.get_game_rows();
						let image = image.into_slint_image();

						for index in indices {
							if !ui_scheduler.should_apply(index) {
								continue;
							}

							let Some(mut row) = rows.row_data(index) else {
								continue;
							};

							row.thumbnail = image.clone();
							row.has_thumbnail = true;
							rows.set_row_data(index, row);
						}
					});

					if sent.is_err() {
						scheduler.finish_thumbnail(&job, ThumbnailOutcome::Failed);
					}
				}
				ImageFetchOutcome::Cancelled => {
					scheduler.finish_thumbnail(&job, ThumbnailOutcome::Cancelled);
				}
				ImageFetchOutcome::Failed => {
					scheduler.finish_thumbnail(&job, ThumbnailOutcome::Failed);
				}
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

	for _ in 0..IMAGE_WORKER_COUNT {
		spawn_image_worker(scheduler.clone(), window.clone());
	}

	scheduler
}
