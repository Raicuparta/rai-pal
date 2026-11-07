use std::{
	collections::HashMap,
	fmt::Display,
	sync::{
		LazyLock, Mutex,
		atomic::{AtomicI32, Ordering},
	},
	thread,
	time::Duration,
};

use rai_pal_core::result::LogErrExt;
use slint::{ComponentHandle, Weak};

use crate::{ActionState, ActionStates, MainWindow, notifications};

const SUCCESS_DURATION: Duration = Duration::from_secs(1);
const ERROR_DURATION: Duration = Duration::from_secs(2);

static STATES: LazyLock<Mutex<HashMap<String, ActionState>>> =
	LazyLock::new(|| Mutex::new(HashMap::new()));
static VERSION: AtomicI32 = AtomicI32::new(1);

pub fn lookup(id: &str) -> ActionState {
	STATES
		.lock()
		.ok_or_log("Failed to lock action states")
		.and_then(|states| states.get(id).copied())
		.unwrap_or(ActionState::Idle)
}

pub fn start(window: &Weak<MainWindow>, id: impl Into<String>) {
	let window = window.clone();
	let id = id.into();

	crate::runtime::invoke_on_ui(move || {
		if let Some(window) = window.upgrade() {
			set(&window, &id, ActionState::Loading);
		}
	});
}

pub fn finish<E: Display>(
	window: &Weak<MainWindow>,
	id: impl Into<String>,
	result: std::result::Result<(), E>,
	error_context: impl Into<String>,
) {
	let id = id.into();

	match result {
		Ok(()) => {
			set_async(window, &id, ActionState::Success);
			schedule_reset(window.clone(), id, ActionState::Success, SUCCESS_DURATION);
		}
		Err(err) => {
			let error_context = error_context.into();
			let message = if error_context.is_empty() {
				err.to_string()
			} else {
				format!("{error_context}: {err}")
			};

			notifications::error_async(window, message);
			set_async(window, &id, ActionState::Error);
			schedule_reset(window.clone(), id, ActionState::Error, ERROR_DURATION);
		}
	}
}

fn set_async(window: &Weak<MainWindow>, id: &str, state: ActionState) {
	let window = window.clone();
	let id = id.to_string();

	crate::runtime::invoke_on_ui(move || {
		if let Some(window) = window.upgrade() {
			set(&window, &id, state);
		}
	});
}

fn schedule_reset(window: Weak<MainWindow>, id: String, state: ActionState, duration: Duration) {
	thread::spawn(move || {
		thread::sleep(duration);

		crate::runtime::invoke_on_ui(move || {
			let Some(window) = window.upgrade() else {
				return;
			};

			if lookup(&id) == state {
				set(&window, &id, ActionState::Idle);
			}
		});
	});
}

fn set(window: &MainWindow, id: &str, state: ActionState) {
	{
		let Some(mut states) = STATES.lock().ok_or_log("Failed to lock action states") else {
			return;
		};

		if state == ActionState::Idle {
			states.remove(id);
		} else {
			states.insert(id.to_string(), state);
		}
	}

	let version = VERSION.fetch_add(1, Ordering::Relaxed);
	window.global::<ActionStates>().set_version(version);
}
