use std::{
	collections::{HashSet, VecDeque},
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

#[derive(Default)]
struct SchedulerState {
	queue: VecDeque<FetchRequest>,
	queued: HashSet<usize>,
	in_flight: HashSet<usize>,
	loaded: HashSet<usize>,
	visible_first: usize,
	visible_last: usize,
	has_visible_range: bool,
}

#[derive(Default)]
pub struct FetchScheduler {
	state: Mutex<SchedulerState>,
	condvar: Condvar,
	generation: AtomicU64,
	thumbnails_enabled: AtomicBool,
}

impl FetchScheduler {
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

		if !state.has_visible_range {
			return true;
		}

		let start = state.visible_first.saturating_sub(VISIBLE_MARGIN);
		let end = state.visible_last.saturating_add(VISIBLE_MARGIN);

		index >= start && index <= end
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
		}

		generation
	}

	pub fn generation(&self) -> u64 {
		self.generation.load(Ordering::SeqCst)
	}

	pub fn set_thumbnails_enabled(&self, enabled: bool) {
		self.thumbnails_enabled.store(enabled, Ordering::SeqCst);
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
