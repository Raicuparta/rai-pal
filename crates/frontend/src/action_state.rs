use std::{
	collections::HashMap,
	fmt::Display,
	hash::{Hash, Hasher},
	sync::{
		LazyLock, Mutex,
		atomic::{AtomicI32, Ordering},
	},
	time::Duration,
};

use rai_pal_backend::result::LogErrExt;
use slint::{ComponentHandle, SharedString, Weak};

use crate::{
	ActionKey, ActionKind, ActionState, ActionStates, MainWindow, MessageContext, notifications,
};

const SUCCESS_DURATION: Duration = Duration::from_secs(1);
const ERROR_DURATION: Duration = Duration::from_secs(2);

static STATES: LazyLock<Mutex<HashMap<ActionKey, ActionState>>> =
	LazyLock::new(|| Mutex::new(HashMap::new()));
static VERSION: AtomicI32 = AtomicI32::new(1);

impl Eq for ActionKind {}

impl Hash for ActionKind {
	fn hash<H: Hasher>(&self, state: &mut H) {
		std::mem::discriminant(self).hash(state);
	}
}

impl Eq for ActionKey {}

impl Hash for ActionKey {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.kind.hash(state);
		self.target.hash(state);
	}
}

pub fn key(kind: ActionKind, target: impl Into<SharedString>) -> ActionKey {
	ActionKey {
		kind,
		target: target.into(),
	}
}

pub fn lookup(key: &ActionKey) -> ActionState {
	STATES
		.lock()
		.ok_or_log("Failed to lock action states")
		.and_then(|states| states.get(key).copied())
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
	context: MessageContext,
) {
	match result {
		Ok(()) => {
			set_async(window, &key, ActionState::Success);
			schedule_reset(window.clone(), key, ActionState::Success, SUCCESS_DURATION);
		}
		Err(err) => {
			notifications::error_async(
				window,
				notifications::data_with_subject(context, key.target.clone(), err),
			);
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
	crate::runtime::invoke_on_ui(move || {
		slint::Timer::single_shot(duration, move || {
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

		if state == ActionState::Idle {
			states.remove(key);
		} else {
			states.insert(key.clone(), state);
		}
	}

	let version = VERSION.fetch_add(1, Ordering::Relaxed);
	window.global::<ActionStates>().set_version(version);
}
