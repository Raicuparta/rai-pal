use std::{
	collections::{HashMap, HashSet, VecDeque},
	sync::{
		Condvar, Mutex,
		atomic::{AtomicBool, AtomicU64, Ordering},
	},
};

use slint::{Model, ModelRc};

use crate::{GameRowData, model::reset_row};

const VISIBLE_MARGIN: usize = 8;

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

	fn thumbnail_job_wanted(state: &SchedulerState, job: &ThumbnailJob) -> bool {
		state
			.thumbnail_subscribers
			.get(job)
			.is_some_and(|indices| {
				indices
					.iter()
					.any(|index| Self::index_in_visible_margin(state, *index))
			})
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
				while let Some(job) = state.thumbnail_queue.pop_front() {
					state.thumbnail_queued.remove(&job);

					if job.generation != self.generation() {
						continue;
					}

					if !Self::thumbnail_job_wanted(&state, &job) {
						state.thumbnail_subscribers.remove(&job);
						continue;
					}

					state.thumbnail_in_flight.insert(job.clone());
					return Some(job);
				}
			} else {
				state.thumbnail_queue.clear();
				state.thumbnail_queued.clear();
				state.thumbnail_subscribers.clear();
			}

			state = self.condvar.wait(state).ok()?;
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
			ThumbnailOutcome::Failed => {
				state.thumbnail_subscribers.remove(job);
				Vec::new()
			}
			ThumbnailOutcome::Cancelled => {
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
	rows: &ModelRc<GameRowData>,
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

pub fn enqueue_index(scheduler: &FetchScheduler, rows: &ModelRc<GameRowData>, index: usize) {
	if let Some(request) = make_request(rows, index, scheduler.generation()) {
		scheduler.enqueue(request);
	}
}

pub fn schedule_visible_range(
	scheduler: &FetchScheduler,
	rows: &ModelRc<GameRowData>,
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
