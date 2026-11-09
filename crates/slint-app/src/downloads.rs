use slint::ComponentHandle as _;
use std::{
	rc::Rc,
	sync::{Arc, Mutex},
	thread,
	time::{Duration, Instant},
};

use rai_pal_core::{
	mods::manifest::{InstallStep, InstallStepKind},
	progress_status::ProgressStatus,
	result::LogErrExt,
};

use crate::{DownloadItemData, DownloadPhase, MainWindow};

const PUBLISH_INTERVAL: Duration = Duration::from_millis(100);
const FINISHED_RETENTION: Duration = Duration::from_secs(5);

#[derive(Clone, Copy, PartialEq)]
enum Phase {
	Pending,
	InProgress,
	Finished,
	Failed,
}

#[derive(Clone)]
struct Item {
	id: String,
	name: String,
	progress: f64,
	phase: Phase,
}

#[derive(Clone)]
pub struct Tracker {
	items: Arc<Mutex<Vec<Item>>>,
	last_publish: Arc<Mutex<Instant>>,
	window: slint::Weak<MainWindow>,
}

impl Tracker {
	pub fn new(window: slint::Weak<MainWindow>) -> Self {
		Self {
			items: Arc::new(Mutex::new(Vec::new())),
			last_publish: Arc::new(Mutex::new(Instant::now())),
			window,
		}
	}

	pub fn pending_steps(&self, steps: &[InstallStep]) {
		if let Some(mut items) = self.items.lock().ok_or_log("Failed to lock download items") {
			for step in steps {
				upsert(
					&mut items,
					&step.id(),
					&step_name(step),
					0.0,
					Phase::Pending,
				);
			}
		}

		self.publish();
	}

	pub fn record(&self, status: ProgressStatus) {
		match status {
			ProgressStatus::Pending { id, name } => {
				if let Some(mut items) =
					self.items.lock().ok_or_log("Failed to lock download items")
				{
					upsert(&mut items, &id, &name, 0.0, Phase::Pending);
				}
				self.publish();
			}
			ProgressStatus::InProgress { id, progress } => {
				if let Some(mut items) =
					self.items.lock().ok_or_log("Failed to lock download items")
				{
					let progress = progress * 100.0;
					update_existing(&mut items, &id, Some(progress), Phase::InProgress);
				}
				self.publish_if_due();
			}
			ProgressStatus::Finished { id } => {
				if let Some(mut items) =
					self.items.lock().ok_or_log("Failed to lock download items")
				{
					update_existing(&mut items, &id, Some(100.0), Phase::Finished);
				}
				self.publish();
				self.schedule_cleanup();
			}
			ProgressStatus::Failed { id, .. } => {
				if let Some(mut items) =
					self.items.lock().ok_or_log("Failed to lock download items")
				{
					update_existing(&mut items, &id, None, Phase::Failed);
				}
				self.publish();
			}
		}
	}

	pub fn finish_steps(&self, steps: &[InstallStep]) {
		if let Some(mut items) = self.items.lock().ok_or_log("Failed to lock download items") {
			for step in steps {
				upsert(
					&mut items,
					&step.id(),
					&step_name(step),
					100.0,
					Phase::Finished,
				);
			}
		}

		self.publish();
		self.schedule_cleanup();
	}

	pub fn fail_steps(&self, steps: &[InstallStep]) {
		if let Some(mut items) = self.items.lock().ok_or_log("Failed to lock download items") {
			for step in steps {
				update_existing(&mut items, &step.id(), None, Phase::Failed);
			}
		}

		self.publish();
	}

	pub fn clear_completed(&self) {
		if let Some(mut items) = self.items.lock().ok_or_log("Failed to lock download items") {
			items.retain(|item| item.phase != Phase::Finished && item.phase != Phase::Failed);
		}

		self.publish();
	}

	fn publish_if_due(&self) {
		let Some(mut last_publish) = self
			.last_publish
			.lock()
			.ok_or_log("Failed to lock download publish timer")
		else {
			return;
		};

		if last_publish.elapsed() < PUBLISH_INTERVAL {
			return;
		}

		*last_publish = Instant::now();
		drop(last_publish);

		self.publish();
	}

	fn publish(&self) {
		let items = self
			.items
			.lock()
			.ok_or_log("Failed to lock download items")
			.map(|items| items.clone())
			.unwrap_or_default();

		if let Some(mut last_publish) = self
			.last_publish
			.lock()
			.ok_or_log("Failed to lock download publish timer")
		{
			*last_publish = Instant::now();
		}

		let window = self.window.clone();

		crate::runtime::invoke_on_ui(move || {
			let Some(window) = window.upgrade() else {
				return;
			};

			let overall_progress = if items.is_empty() {
				0.0
			} else {
				let total = items
					.iter()
					.map(|item| match item.phase {
						Phase::Finished | Phase::Failed => 100.0,
						_ => item.progress,
					})
					.sum::<f64>();

				(total / items.len() as f64 / 100.0) as f32
			};

			let rows = items
				.into_iter()
				.map(|item| DownloadItemData {
					id: item.id.into(),
					name: item.name.into(),
					progress: item.progress as f32,
					phase: match item.phase {
						Phase::Pending => DownloadPhase::Pending,
						Phase::InProgress => DownloadPhase::InProgress,
						Phase::Finished => DownloadPhase::Finished,
						Phase::Failed => DownloadPhase::Failed,
					},
				})
				.collect::<Vec<_>>();

			window
				.global::<crate::DownloadsState>()
				.set_overall_progress(overall_progress);
			window
				.global::<crate::DownloadsState>()
				.set_items(Rc::new(slint::VecModel::from(rows)).into());
		});
	}

	fn schedule_cleanup(&self) {
		let items = self.items.clone();
		let window = self.window.clone();
		let last_publish = self.last_publish.clone();

		thread::spawn(move || {
			thread::sleep(FINISHED_RETENTION);

			let removed = items
				.lock()
				.ok_or_log("Failed to lock download items")
				.map(|mut items| {
					let has_active = items
						.iter()
						.any(|item| matches!(item.phase, Phase::Pending | Phase::InProgress));

					if has_active {
						return false;
					}

					let before = items.len();
					items.retain(|item| item.phase != Phase::Finished);
					items.len() != before
				})
				.unwrap_or(false);

			if !removed {
				return;
			}

			let tracker = Tracker {
				items,
				last_publish,
				window,
			};

			tracker.publish();
		});
	}
}

fn step_name(step: &InstallStep) -> String {
	match step.kind {
		InstallStepKind::Download => format!("Download {}", step.mod_id),
		InstallStepKind::Extract => format!("Extract {}", step.mod_id),
	}
}

fn upsert(items: &mut Vec<Item>, id: &str, name: &str, progress: f64, phase: Phase) {
	if let Some(item) = items.iter_mut().find(|item| item.id == id) {
		item.name = name.to_string();
		item.progress = progress;
		item.phase = phase;
		return;
	}

	items.push(Item {
		id: id.to_string(),
		name: name.to_string(),
		progress,
		phase,
	});
}

fn update_existing(items: &mut Vec<Item>, id: &str, progress: Option<f64>, phase: Phase) {
	if let Some(item) = items.iter_mut().find(|item| item.id == id) {
		if let Some(progress) = progress {
			item.progress = progress;
		}
		item.phase = phase;
		return;
	}

	items.push(Item {
		id: id.to_string(),
		name: id.to_string(),
		progress: progress.unwrap_or_default(),
		phase,
	});
}
