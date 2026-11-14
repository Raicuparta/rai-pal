use slint::ComponentHandle as _;
use std::{
	sync::{Arc, Mutex, mpsc},
	thread,
	time::Duration,
};

use rai_pal_backend::{
	game::Game,
	http,
	local_database::{app_database::DbMutex, game_database::GameDatabase},
	result::{ContextResult, LogErrExt, Result as BackendResult},
};

use slint::Model;

use crate::{
	MainWindow, image_cache,
	model::{apply_cached_thumbnail, apply_game_to_row},
	runtime,
	scheduler::{FetchRequest, FetchScheduler, ThumbnailJob, ThumbnailOutcome},
};

const WORKER_COUNT: usize = 4;
const IMAGE_DOWNLOAD_WORKERS: usize = 6;
const IMAGE_DECODE_WORKERS: usize = 2;
const IMAGE_REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

pub struct FetchedImage {
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

struct DecodeJob {
	job: ThumbnailJob,
	bytes: Vec<u8>,
	from_cache: bool,
}

enum ImageBytesOutcome {
	Fetched(Vec<u8>),
	Cancelled,
	Failed,
}

pub async fn fetch_image(url: &str, max_width: u32, max_height: u32) -> Option<FetchedImage> {
	match download_image_bytes(url, || true).await {
		Ok(Some(bytes)) => decode_image(&bytes, max_width, max_height),
		Ok(None) => None,
		Err(err) => {
			log::warn!("{err}");
			None
		}
	}
}

fn fetch_image_bytes_cancellable<F>(url: &str, should_continue: F) -> ImageBytesOutcome
where
	F: FnMut() -> bool,
{
	match runtime::block_on(download_image_bytes(url, should_continue)) {
		Ok(Some(bytes)) => ImageBytesOutcome::Fetched(bytes),
		Ok(None) => ImageBytesOutcome::Cancelled,
		Err(err) => {
			log::warn!("{err}");
			ImageBytesOutcome::Failed
		}
	}
}

async fn download_image_bytes<F>(
	url: &str,
	mut should_continue: F,
) -> BackendResult<Option<Vec<u8>>>
where
	F: FnMut() -> bool,
{
	if !should_continue() {
		return Ok(None);
	}

	let response = http::CLIENT
		.get(url)
		.timeout(IMAGE_REQUEST_TIMEOUT)
		.send()
		.await
		.with_context(|| format!("failed to request image `{url}`"))?
		.error_for_status()
		.with_context(|| format!("image request for `{url}` failed"))?;

	let bytes = response
		.bytes()
		.await
		.with_context(|| format!("failed to download image `{url}`"))?;

	if !should_continue() {
		return Ok(None);
	}

	Ok(Some(bytes.to_vec()))
}

fn decode_image(bytes: &[u8], max_width: u32, max_height: u32) -> Option<FetchedImage> {
	let image = image::load_from_memory(bytes).ok_or_log("Failed to decode image")?;
	let image = image.thumbnail(max_width, max_height).to_rgba8();

	Some(FetchedImage {
		width: image.width(),
		height: image.height(),
		pixels: image.into_raw(),
	})
}

fn fetch_game(database: &DbMutex, request: &FetchRequest) -> FetchResult {
	match database.get_game(&request.provider_id, &request.game_id) {
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
			let ui_scheduler = Arc::clone(&scheduler);
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

				let rows = window.global::<crate::GamesList>().get_rows();
				let Some(mut row) = rows.row_data(result.index) else {
					return;
				};

				match result.game {
					Ok(game) => {
						let thumbnail_url = game.thumbnail_url.clone();
						apply_game_to_row(&mut row, &game);
						let thumbnail_cached = apply_cached_thumbnail(&mut row);
						rows.set_row_data(result.index, row);
						ui_scheduler.mark_loaded(result.index);

						if !thumbnail_cached
							&& let Some(url) = thumbnail_url.filter(|url| !url.is_empty())
						{
							ui_scheduler.enqueue_thumbnail(result.index, result.generation, url);
						}
					}
					Err(title) => {
						row.title = title.into();
						row.loaded = true;
						rows.set_row_data(result.index, row);
						ui_scheduler.mark_loaded(result.index);
					}
				}
			});

			if let Err(err) = sent {
				log::warn!("Failed to send game fetch result to UI: {err}");
				scheduler.finish(index);
			}
		}
	});
}

fn spawn_download_worker(scheduler: Arc<FetchScheduler>, decode_tx: mpsc::Sender<DecodeJob>) {
	thread::spawn(move || {
		while let Some(job) = scheduler.take_next_thumbnail() {
			let cancel_scheduler = Arc::clone(&scheduler);
			let cancel_job = job.clone();
			let mut should_continue = move || cancel_scheduler.thumbnail_wanted(&cancel_job);

			let width = image_cache::THUMBNAIL_WIDTH;
			let height = image_cache::THUMBNAIL_HEIGHT;

			let (bytes, from_cache) =
				if let Some(bytes) = image_cache::load(&job.url, width, height) {
					(bytes, true)
				} else {
					match fetch_image_bytes_cancellable(&job.url, &mut should_continue) {
						ImageBytesOutcome::Fetched(bytes) => (bytes, false),
						ImageBytesOutcome::Cancelled => {
							scheduler.finish_thumbnail(&job, ThumbnailOutcome::Cancelled);
							continue;
						}
						ImageBytesOutcome::Failed => {
							scheduler.finish_thumbnail(&job, ThumbnailOutcome::Failed);
							continue;
						}
					}
				};

			let decode_job = DecodeJob {
				job,
				bytes,
				from_cache,
			};

			if let Err(err) = decode_tx.send(decode_job) {
				log::warn!("Failed to send thumbnail bytes to decode workers: {err}");
				scheduler.finish_thumbnail(&err.0.job, ThumbnailOutcome::Failed);
			}
		}
	});
}

fn spawn_decode_worker(
	scheduler: Arc<FetchScheduler>,
	window: slint::Weak<MainWindow>,
	decode_rx: Arc<Mutex<mpsc::Receiver<DecodeJob>>>,
) {
	thread::spawn(move || {
		loop {
			let decode_job = {
				let receiver = match decode_rx.lock() {
					Ok(receiver) => receiver,
					Err(err) => {
						log::error!("Failed to lock thumbnail decode queue: {err}");
						return;
					}
				};

				receiver.recv()
			};

			let Ok(DecodeJob {
				job,
				bytes,
				from_cache,
			}) = decode_job
			else {
				return;
			};

			if !scheduler.thumbnail_wanted(&job) {
				scheduler.finish_thumbnail(&job, ThumbnailOutcome::Cancelled);
				continue;
			}

			let width = image_cache::THUMBNAIL_WIDTH;
			let height = image_cache::THUMBNAIL_HEIGHT;

			let Some(image) = decode_image(&bytes, width, height) else {
				if from_cache {
					image_cache::remove(&job.url, width, height);
				}

				scheduler.finish_thumbnail(&job, ThumbnailOutcome::Failed);
				continue;
			};

			if !from_cache {
				image_cache::store(&job.url, width, height, &bytes);
			}

			let window = window.clone();
			let ui_scheduler = Arc::clone(&scheduler);
			let ui_job = job.clone();

			let sent = slint::invoke_from_event_loop(move || {
				let image = image.into_slint_image();
				image_cache::store_memory(&ui_job.url, image.clone());

				let indices = ui_scheduler.finish_thumbnail(&ui_job, ThumbnailOutcome::Success);

				if ui_job.generation != ui_scheduler.generation() || indices.is_empty() {
					return;
				}

				let Some(window) = window.upgrade() else {
					return;
				};

				let rows = window.global::<crate::GamesList>().get_rows();

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

			if let Err(err) = sent {
				log::warn!("Failed to send thumbnail fetch result to UI: {err}");
				scheduler.finish_thumbnail(&job, ThumbnailOutcome::Failed);
			}
		}
	});
}

pub fn spawn_fetchers(
	database: &Arc<DbMutex>,
	window: &slint::Weak<MainWindow>,
) -> Arc<FetchScheduler> {
	let scheduler = Arc::new(FetchScheduler::default());

	thread::spawn(image_cache::prune);

	for _ in 0..WORKER_COUNT {
		spawn_worker(Arc::clone(&scheduler), Arc::clone(database), window.clone());
	}

	let (decode_tx, decode_rx) = mpsc::channel::<DecodeJob>();
	let decode_rx = Arc::new(Mutex::new(decode_rx));

	for _ in 0..IMAGE_DECODE_WORKERS {
		spawn_decode_worker(
			Arc::clone(&scheduler),
			window.clone(),
			Arc::clone(&decode_rx),
		);
	}

	for _ in 0..IMAGE_DOWNLOAD_WORKERS {
		spawn_download_worker(Arc::clone(&scheduler), decode_tx.clone());
	}

	scheduler
}
