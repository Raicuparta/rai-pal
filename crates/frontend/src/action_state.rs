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

use rai_pal_backend::result::LogErrExt;
use slint::{ComponentHandle, SharedString, Weak};

use crate::{ActionKey, ActionKind, ActionState, ActionStates, MainWindow, notifications};

const SUCCESS_DURATION: Duration = Duration::from_secs(1);
const ERROR_DURATION: Duration = Duration::from_secs(2);

static STATES: LazyLock<Mutex<HashMap<String, ActionState>>> =
	LazyLock::new(|| Mutex::new(HashMap::new()));
static VERSION: AtomicI32 = AtomicI32::new(1);

pub fn key(kind: ActionKind, target: impl Into<SharedString>) -> ActionKey {
	ActionKey {
		kind,
		target: target.into(),
	}
}

fn storage_key(key: &ActionKey) -> String {
	format!("{:?}:{}", key.kind, key.target)
}

pub fn lookup(key: &ActionKey) -> ActionState {
	STATES
		.lock()
		.ok_or_log("Failed to lock action states")
		.and_then(|states| states.get(&storage_key(key)).copied())
		.unwrap_or(ActionState::Idle)
}

pub fn start(window: &Weak<MainWindow>, key: ActionKey) {
	let window = window.clone();

	crate::runtime::invoke_on_ui(move || {
		if let Some(window) = window.upgrade() {
			set(&window, &key, ActionState::Loading);
		}
	});
}

pub fn finish<E: Display>(
	window: &Weak<MainWindow>,
	key: ActionKey,
	result: std::result::Result<(), E>,
	error_context: impl Into<String>,
) {
	match result {
		Ok(()) => {
			set_async(window, &key, ActionState::Success);
			schedule_reset(window.clone(), key, ActionState::Success, SUCCESS_DURATION);
		}
		Err(err) => {
			let error_context = error_context.into();
			let message = if error_context.is_empty() {
				err.to_string()
			} else {
				format!("{error_context}: {err}")
			};

			notifications::error_async(window, message);
			set_async(window, &key, ActionState::Error);
			schedule_reset(window.clone(), key, ActionState::Error, ERROR_DURATION);
		}
	}
}

fn set_async(window: &Weak<MainWindow>, key: &ActionKey, state: ActionState) {
	let window = window.clone();
	let key = key.clone();

	crate::runtime::invoke_on_ui(move || {
		if let Some(window) = window.upgrade() {
			set(&window, &key, state);
		}
	});
}

fn schedule_reset(
	window: Weak<MainWindow>,
	key: ActionKey,
	state: ActionState,
	duration: Duration,
) {
	thread::spawn(move || {
		thread::sleep(duration);

		crate::runtime::invoke_on_ui(move || {
			let Some(window) = window.upgrade() else {
				return;
			};

			if lookup(&key) == state {
				set(&window, &key, ActionState::Idle);
			}
		});
	});
}

fn set(window: &MainWindow, key: &ActionKey, state: ActionState) {
	{
		let Some(mut states) = STATES.lock().ok_or_log("Failed to lock action states") else {
			return;
		};

		let storage_key = storage_key(key);
		if state == ActionState::Idle {
			states.remove(&storage_key);
		} else {
			states.insert(storage_key, state);
		}
	}

	let version = VERSION.fetch_add(1, Ordering::Relaxed);
	window.global::<ActionStates>().set_version(version);
}
