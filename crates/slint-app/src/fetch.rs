use std::{
	io::Read,
	sync::{Arc, LazyLock},
	thread,
	time::Duration,
};

use rai_pal_core::{
	game::Game,
	game_providers::game_provider::GameProviderId,
	local_database::{app_database::DbMutex, game_database::GameDatabase},
	result::LogErrExt,
};

use slint::Model;

use crate::{
	MainWindow, image_cache,
	model::apply_game_to_row,
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
	game: Result<Game, String>,
}

pub(crate) enum ImageFetchOutcome {
	Fetched(FetchedImage),
	Cancelled,
	Failed,
}

enum ImageBytesOutcome {
	Fetched(Vec<u8>),
	Cancelled,
	Failed,
}

pub(crate) fn fetch_image(url: &str, max_width: u32, max_height: u32) -> Option<FetchedImage> {
	match fetch_image_bytes_cancellable(url, || true) {
		ImageBytesOutcome::Fetched(bytes) => decode_image(&bytes, max_width, max_height),
		ImageBytesOutcome::Cancelled | ImageBytesOutcome::Failed => None,
	}
}

fn fetch_image_bytes_cancellable<F>(url: &str, mut should_continue: F) -> ImageBytesOutcome
where
	F: FnMut() -> bool,
{
	let Some(response) = HTTP_CLIENT
		.get(url)
		.send()
		.ok_or_log("Failed to request image")
	else {
		return ImageBytesOutcome::Failed;
	};
	let Some(mut response) = response
		.error_for_status()
		.ok_or_log("Image request failed")
	else {
		return ImageBytesOutcome::Failed;
	};

	let mut bytes = Vec::new();
	let mut chunk = [0u8; 16 * 1024];

	loop {
		if !should_continue() {
			return ImageBytesOutcome::Cancelled;
		}

		let Some(length) = response
			.read(&mut chunk)
			.ok_or_log("Failed to read image response")
		else {
			return ImageBytesOutcome::Failed;
		};
		if length == 0 {
			break;
		}
		bytes.extend_from_slice(&chunk[..length]);
	}

	if !should_continue() {
		return ImageBytesOutcome::Cancelled;
	}

	ImageBytesOutcome::Fetched(bytes)
}

fn decode_image(bytes: &[u8], max_width: u32, max_height: u32) -> Option<FetchedImage> {
	let image = image::load_from_memory(bytes).ok()?;
	let image = image.thumbnail(max_width, max_height).to_rgba8();

	Some(FetchedImage {
		width: image.width(),
		height: image.height(),
		pixels: image.into_raw(),
	})
}

fn fetch_thumbnail_cached<F>(
	url: &str,
	max_width: u32,
	max_height: u32,
	mut should_continue: F,
) -> ImageFetchOutcome
where
	F: FnMut() -> bool,
{
	if !should_continue() {
		return ImageFetchOutcome::Cancelled;
	}

	if let Some(bytes) = image_cache::load(url, max_width, max_height) {
		if !should_continue() {
			return ImageFetchOutcome::Cancelled;
		}

		if let Some(image) = decode_image(&bytes, max_width, max_height) {
			return ImageFetchOutcome::Fetched(image);
		}

		image_cache::remove(url, max_width, max_height);
	}

	let bytes = match fetch_image_bytes_cancellable(url, &mut should_continue) {
		ImageBytesOutcome::Fetched(bytes) => bytes,
		ImageBytesOutcome::Cancelled => return ImageFetchOutcome::Cancelled,
		ImageBytesOutcome::Failed => return ImageFetchOutcome::Failed,
	};

	let Some(image) = decode_image(&bytes, max_width, max_height) else {
		return ImageFetchOutcome::Failed;
	};

	image_cache::store(url, max_width, max_height, &bytes);

	ImageFetchOutcome::Fetched(image)
}

fn fetch_game(database: &DbMutex, request: &FetchRequest) -> FetchResult {
	let provider_id = match request.provider_id.parse::<GameProviderId>() {
		Ok(provider_id) => provider_id,
		Err(err) => {
			log::error!("Invalid provider id {}: {err}", request.provider_id);
			return FetchResult {
				index: request.index,
				generation: request.generation,
				game: Err(format!("{}: {}", request.provider_id, request.game_id)),
			};
		}
	};

	match database.get_game(&provider_id, &request.game_id) {
		Ok(game) => FetchResult {
			index: request.index,
			generation: request.generation,
			game: Ok(game),
		},
		Err(err) => {
			log::error!(
				"Failed to load game {}:{}: {err}",
				request.provider_id,
				request.game_id
			);
			FetchResult {
				index: request.index,
				generation: request.generation,
				game: Err(format!("{}: {}", request.provider_id, request.game_id)),
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

				let thumbnail_url = match &result.game {
					Ok(game) => game.thumbnail_url.clone(),
					Err(_) => None,
				};

				match result.game {
					Ok(game) => apply_game_to_row(&mut row, &game),
					Err(title) => {
						row.title = title.into();
						row.loaded = true;
					}
				}

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
			let outcome = fetch_thumbnail_cached(
				&job.url,
				image_cache::THUMBNAIL_WIDTH,
				image_cache::THUMBNAIL_HEIGHT,
				move || cancel_scheduler.thumbnail_wanted(&cancel_job),
			);

			match outcome {
				ImageFetchOutcome::Fetched(image) => {
					let window = window.clone();
					let ui_scheduler = scheduler.clone();
					let ui_job = job.clone();

					let sent = slint::invoke_from_event_loop(move || {
						let indices =
							ui_scheduler.finish_thumbnail(&ui_job, ThumbnailOutcome::Success);

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
					thread::sleep(Duration::from_secs(1));
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

	thread::spawn(image_cache::prune);

	for _ in 0..WORKER_COUNT {
		spawn_worker(scheduler.clone(), database.clone(), window.clone());
	}

	for _ in 0..IMAGE_WORKER_COUNT {
		spawn_image_worker(scheduler.clone(), window.clone());
	}

	scheduler
}
