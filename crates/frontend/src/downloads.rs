use slint::ComponentHandle as _;
use std::{
	rc::Rc,
	sync::{Arc, Mutex},
	thread,
	time::{Duration, Instant},
};

use rai_pal_backend::{
	mods::manifest::{InstallStep, InstallStepKind},
	progress_status::ProgressStatus,
	result::LogErrExt,
};

use crate::{DownloadItemData, DownloadItemKind, DownloadPhase, MainWindow};

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
	kind: DownloadItemKind,
	mod_id: String,
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
				set_item(&mut items, step, Some(0.0), Phase::Pending);
			}
		}

		self.publish();
	}

	pub fn record(&self, status: ProgressStatus) {
		if let Some(mut items) = self.items.lock().ok_or_log("Failed to lock download items") {
			let progress = status.progress * 100.0;
			set_item(&mut items, &status.step, Some(progress), Phase::InProgress);
		}
		self.publish_if_due();
	}

	pub fn finish_steps(&self, steps: &[InstallStep]) {
		if let Some(mut items) = self.items.lock().ok_or_log("Failed to lock download items") {
			for step in steps {
				set_item(&mut items, step, Some(100.0), Phase::Finished);
			}
		}

		self.publish();
		self.schedule_cleanup();
	}

	pub fn fail_steps(&self, steps: &[InstallStep]) {
		if let Some(mut items) = self.items.lock().ok_or_log("Failed to lock download items") {
			for step in steps {
				set_item(&mut items, step, None, Phase::Failed);
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

				let count = u32::try_from(items.len()).unwrap_or(u32::MAX);
				let average = total / f64::from(count) / 100.0;
				progress_to_f32(average)
			};

			let rows = items
				.into_iter()
				.map(|item| DownloadItemData {
					id: item.id.into(),
					kind: item.kind,
					mod_id: item.mod_id.into(),
					progress: progress_to_f32(item.progress),
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
		let items = Arc::clone(&self.items);
		let window = self.window.clone();
		let last_publish = Arc::clone(&self.last_publish);

		thread::spawn(move || {
			thread::sleep(FINISHED_RETENTION);

			let removed = items
				.lock()
				.ok_or_log("Failed to lock download items")
				.is_some_and(|mut items| {
					let has_active = items
						.iter()
						.any(|item| matches!(item.phase, Phase::Pending | Phase::InProgress));

					if has_active {
						return false;
					}

					let before = items.len();
					items.retain(|item| item.phase != Phase::Finished);
					items.len() != before
				});

			if !removed {
				return;
			}

			let tracker = Self {
				items,
				last_publish,
				window,
			};

			tracker.publish();
		});
	}
}

#[expect(
	clippy::cast_possible_truncation,
	reason = "progress is a normalized fraction for the UI"
)]
const fn progress_to_f32(progress: f64) -> f32 {
	progress as f32
}

const fn item_kind(kind: InstallStepKind) -> DownloadItemKind {
	match kind {
		InstallStepKind::Download => DownloadItemKind::Download,
		InstallStepKind::Extract => DownloadItemKind::Extract,
	}
}

fn set_item(items: &mut Vec<Item>, step: &InstallStep, progress: Option<f64>, phase: Phase) {
	let id = step.id();
	let kind = item_kind(step.kind);

	if let Some(item) = items.iter_mut().find(|item| item.id == id) {
		item.kind = kind;
		item.mod_id.clone_from(&step.mod_id);
		if let Some(progress) = progress {
			item.progress = progress;
		}
		item.phase = phase;
		return;
	}

	items.push(Item {
		id,
		kind,
		mod_id: step.mod_id.clone(),
		progress: progress.unwrap_or_default(),
		phase,
	});
}

pub fn register(window: &MainWindow) {
	window.global::<crate::AppActions>().on_clear_downloads(|| {
		if let Some(handler) = crate::commands::CommandHandler::current() {
			handler.downloads.clear_completed();
		}
	});
}
