use std::{
	collections::{HashMap, HashSet, VecDeque},
	sync::{
		Condvar, Mutex,
		atomic::{AtomicBool, AtomicU64, Ordering},
	},
	time::{Duration, Instant},
};

use slint::{Model, ModelRc};

use crate::{GameRowView, image_cache, model::reset_row};

const VISIBLE_MARGIN: usize = 8;
const THUMBNAIL_VISIBLE_DELAY: Duration = Duration::from_millis(100);

pub struct FetchRequest {
	pub index: usize,
	pub generation: u64,
	pub provider_id: String,
	pub game_id: String,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct ThumbnailJob {
	pub generation: u64,
	pub url: String,
}

pub enum ThumbnailOutcome {
	Success,
	Cancelled,
	Failed,
}

enum ThumbnailEligibility {
	Ready,
	Waiting(Duration),
	NotVisible,
}

#[derive(Default)]
struct SchedulerState {
	queue: VecDeque<FetchRequest>,
	queued: HashSet<usize>,
	in_flight: HashSet<usize>,
	loaded: HashSet<usize>,
	visible_first: usize,
	visible_last: usize,
	has_visible_range: bool,
	thumbnail_queue: VecDeque<ThumbnailJob>,
	thumbnail_queued: HashSet<ThumbnailJob>,
	thumbnail_in_flight: HashSet<ThumbnailJob>,
	thumbnail_subscribers: HashMap<ThumbnailJob, Vec<usize>>,
	visible_since: HashMap<usize, Instant>,
}

#[derive(Default)]
pub struct FetchScheduler {
	state: Mutex<SchedulerState>,
	condvar: Condvar,
	generation: AtomicU64,
	thumbnails_enabled: AtomicBool,
}

impl FetchScheduler {
	fn index_in_visible_margin(state: &SchedulerState, index: usize) -> bool {
		if !state.has_visible_range {
			return true;
		}

		let start = state.visible_first.saturating_sub(VISIBLE_MARGIN);
		let end = state.visible_last.saturating_add(VISIBLE_MARGIN);

		index >= start && index <= end
	}

	fn index_is_visible(state: &SchedulerState, index: usize) -> bool {
		state.has_visible_range && index >= state.visible_first && index <= state.visible_last
	}

	fn thumbnail_job_wanted(state: &SchedulerState, job: &ThumbnailJob) -> bool {
		state.thumbnail_subscribers.get(job).is_some_and(|indices| {
			indices
				.iter()
				.any(|index| Self::index_in_visible_margin(state, *index))
		})
	}

	fn thumbnail_job_eligibility(
		state: &SchedulerState,
		job: &ThumbnailJob,
		now: Instant,
	) -> ThumbnailEligibility {
		let Some(indices) = state.thumbnail_subscribers.get(job) else {
			return ThumbnailEligibility::NotVisible;
		};

		let cached = image_cache::contains(
			&job.url,
			image_cache::THUMBNAIL_WIDTH,
			image_cache::THUMBNAIL_HEIGHT,
		);
		let mut waiting = None;
		let mut has_visible_subscriber = false;

		for index in indices {
			if !Self::index_is_visible(state, *index) {
				continue;
			}

			has_visible_subscriber = true;

			if cached {
				return ThumbnailEligibility::Ready;
			}

			let visible_since = state.visible_since.get(index).copied().unwrap_or(now);
			let elapsed = now.saturating_duration_since(visible_since);

			if elapsed >= THUMBNAIL_VISIBLE_DELAY {
				return ThumbnailEligibility::Ready;
			}

			let remaining = THUMBNAIL_VISIBLE_DELAY - elapsed;
			waiting = Some(waiting.map_or(remaining, |current: Duration| current.min(remaining)));
		}

		if has_visible_subscriber {
			ThumbnailEligibility::Waiting(waiting.unwrap_or(THUMBNAIL_VISIBLE_DELAY))
		} else {
			ThumbnailEligibility::NotVisible
		}
	}

	fn enqueue(&self, request: FetchRequest) {
		let Ok(mut state) = self.state.lock() else {
			return;
		};

		if state.in_flight.contains(&request.index) || !state.queued.insert(request.index) {
			return;
		}

		state.queue.push_back(request);
		self.condvar.notify_one();
	}

	pub fn set_visible_requests(&self, requests: Vec<FetchRequest>) {
		let Ok(mut state) = self.state.lock() else {
			return;
		};

		state.queue.clear();
		state.queued.clear();

		for request in requests {
			if state.in_flight.contains(&request.index) {
				continue;
			}

			if state.queued.insert(request.index) {
				state.queue.push_back(request);
			}
		}

		self.condvar.notify_all();
	}

	pub fn set_visible_bounds(&self, first: usize, last: usize) -> Vec<usize> {
		let Ok(mut state) = self.state.lock() else {
			return Vec::new();
		};

		state.visible_first = first;
		state.visible_last = last;
		state.has_visible_range = true;

		let now = Instant::now();
		state
			.visible_since
			.retain(|index, _| *index >= first && *index <= last);

		if first <= last {
			for index in first..=last {
				state.visible_since.entry(index).or_insert(now);
			}
		}

		let start = first.saturating_sub(VISIBLE_MARGIN);
		let end = last.saturating_add(VISIBLE_MARGIN);

		let evicted: Vec<usize> = state
			.loaded
			.iter()
			.copied()
			.filter(|index| *index < start || *index > end)
			.collect();

		for index in &evicted {
			state.loaded.remove(index);
		}

		state.thumbnail_subscribers.retain(|_, indices| {
			indices.retain(|index| *index >= start && *index <= end);
			!indices.is_empty()
		});

		self.condvar.notify_all();

		evicted
	}

	pub fn mark_loaded(&self, index: usize) {
		if let Ok(mut state) = self.state.lock() {
			state.loaded.insert(index);
		}
	}

	pub fn should_apply(&self, index: usize) -> bool {
		let Ok(state) = self.state.lock() else {
			return false;
		};

		Self::index_in_visible_margin(&state, index)
	}

	pub fn enqueue_thumbnail(&self, index: usize, generation: u64, url: String) {
		if !self.thumbnails_enabled() || generation != self.generation() {
			return;
		}

		let job = ThumbnailJob { generation, url };
		let Ok(mut state) = self.state.lock() else {
			return;
		};

		if !Self::index_in_visible_margin(&state, index) {
			return;
		}

		let subscribers = state.thumbnail_subscribers.entry(job.clone()).or_default();

		if !subscribers.contains(&index) {
			subscribers.push(index);
		}

		if state.thumbnail_in_flight.contains(&job) || !state.thumbnail_queued.insert(job.clone()) {
			return;
		}

		state.thumbnail_queue.push_back(job);
		self.condvar.notify_all();
	}

	pub fn take_next_thumbnail(&self) -> Option<ThumbnailJob> {
		let mut state = self.state.lock().ok()?;

		loop {
			if self.thumbnails_enabled() {
				let now = Instant::now();
				let mut next_wait = None;
				let queued_count = state.thumbnail_queue.len();

				for _ in 0..queued_count {
					let Some(job) = state.thumbnail_queue.pop_front() else {
						break;
					};
					state.thumbnail_queued.remove(&job);

					if job.generation != self.generation() {
						continue;
					}

					if !Self::thumbnail_job_wanted(&state, &job) {
						state.thumbnail_subscribers.remove(&job);
						continue;
					}

					match Self::thumbnail_job_eligibility(&state, &job, now) {
						ThumbnailEligibility::Ready => {
							state.thumbnail_in_flight.insert(job.clone());
							return Some(job);
						}
						ThumbnailEligibility::Waiting(duration) => {
							next_wait = Some(
								next_wait
									.map_or(duration, |current: Duration| current.min(duration)),
							);
							state.thumbnail_queued.insert(job.clone());
							state.thumbnail_queue.push_back(job);
						}
						ThumbnailEligibility::NotVisible => {
							state.thumbnail_queued.insert(job.clone());
							state.thumbnail_queue.push_back(job);
						}
					}
				}

				state = if let Some(duration) = next_wait {
					self.condvar.wait_timeout(state, duration).ok()?.0
				} else {
					self.condvar.wait(state).ok()?
				};
			} else {
				state.thumbnail_queue.clear();
				state.thumbnail_queued.clear();
				state.thumbnail_subscribers.clear();
				state = self.condvar.wait(state).ok()?;
			}
		}
	}

	pub fn thumbnail_wanted(&self, job: &ThumbnailJob) -> bool {
		if !self.thumbnails_enabled() || job.generation != self.generation() {
			return false;
		}

		let Ok(state) = self.state.lock() else {
			return false;
		};

		Self::thumbnail_job_wanted(&state, job)
	}

	pub fn finish_thumbnail(&self, job: &ThumbnailJob, outcome: ThumbnailOutcome) -> Vec<usize> {
		let Ok(mut state) = self.state.lock() else {
			return Vec::new();
		};

		state.thumbnail_in_flight.remove(job);

		match outcome {
			ThumbnailOutcome::Success => {
				let mut indices = state.thumbnail_subscribers.remove(job).unwrap_or_default();
				indices.retain(|index| Self::index_in_visible_margin(&state, *index));
				indices
			}
			ThumbnailOutcome::Cancelled | ThumbnailOutcome::Failed => {
				if self.thumbnails_enabled()
					&& job.generation == self.generation()
					&& Self::thumbnail_job_wanted(&state, job)
				{
					if state.thumbnail_queued.insert(job.clone()) {
						state.thumbnail_queue.push_back(job.clone());
						self.condvar.notify_all();
					}
				} else {
					state.thumbnail_subscribers.remove(job);
				}

				Vec::new()
			}
		}
	}

	pub fn take_next(&self) -> Option<FetchRequest> {
		let mut state = self.state.lock().ok()?;

		loop {
			if let Some(request) = state.queue.pop_front() {
				state.queued.remove(&request.index);
				state.in_flight.insert(request.index);
				return Some(request);
			}

			state = self.condvar.wait(state).ok()?;
		}
	}

	pub fn finish(&self, index: usize) {
		if let Ok(mut state) = self.state.lock() {
			state.in_flight.remove(&index);
		}
	}

	pub fn begin_query(&self) -> u64 {
		let generation = self.generation.fetch_add(1, Ordering::SeqCst) + 1;

		if let Ok(mut state) = self.state.lock() {
			state.queue.clear();
			state.queued.clear();
			state.in_flight.clear();
			state.loaded.clear();
			state.has_visible_range = false;
			state.thumbnail_queue.clear();
			state.thumbnail_queued.clear();
			state.thumbnail_in_flight.clear();
			state.thumbnail_subscribers.clear();
			state.visible_since.clear();
		}

		generation
	}

	pub fn generation(&self) -> u64 {
		self.generation.load(Ordering::SeqCst)
	}

	pub fn set_thumbnails_enabled(&self, enabled: bool) {
		self.thumbnails_enabled.store(enabled, Ordering::SeqCst);

		if let Ok(mut state) = self.state.lock() {
			state.thumbnail_queue.clear();
			state.thumbnail_queued.clear();
			state.thumbnail_subscribers.clear();
		}

		self.condvar.notify_all();
	}

	pub fn thumbnails_enabled(&self) -> bool {
		self.thumbnails_enabled.load(Ordering::SeqCst)
	}
}

fn make_request(
	rows: &ModelRc<GameRowView>,
	index: usize,
	generation: u64,
) -> Option<FetchRequest> {
	let row = rows.row_data(index)?;

	if row.loaded {
		return None;
	}

	Some(FetchRequest {
		index,
		generation,
		provider_id: row.provider_id.to_string(),
		game_id: row.game_id.to_string(),
	})
}

pub fn enqueue_index(scheduler: &FetchScheduler, rows: &ModelRc<GameRowView>, index: usize) {
	if let Some(request) = make_request(rows, index, scheduler.generation()) {
		scheduler.enqueue(request);
	}
}

pub fn schedule_visible_range(
	scheduler: &FetchScheduler,
	rows: &ModelRc<GameRowView>,
	first: usize,
	last: usize,
) {
	let count = rows.row_count();

	if count == 0 {
		return;
	}

	let first = first.min(count - 1);
	let last = last.min(count - 1);

	if first > last {
		return;
	}

	let start = first.saturating_sub(VISIBLE_MARGIN);
	let end = last.saturating_add(VISIBLE_MARGIN).min(count - 1);

	for index in scheduler.set_visible_bounds(first, last) {
		if let Some(mut row) = rows.row_data(index) {
			reset_row(&mut row);
			rows.set_row_data(index, row);
		}
	}

	let mut indices = Vec::with_capacity(end - start + 1);
	indices.extend(first..=last);
	indices.extend(last.saturating_add(1)..=end);
	indices.extend((start..first).rev());

	let generation = scheduler.generation();
	let requests = indices
		.into_iter()
		.filter_map(|index| make_request(rows, index, generation))
		.collect();

	scheduler.set_visible_requests(requests);
}
