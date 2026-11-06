use std::{
	rc::Rc,
	sync::atomic::{AtomicI32, Ordering},
};

use slint::Model;

use crate::{MainWindow, NotificationData, NotificationKind};

static NEXT_ID: AtomicI32 = AtomicI32::new(1);

pub fn error(window: &MainWindow, message: impl Into<slint::SharedString>) {
	show(window, message, NotificationKind::Error);
}

pub fn info(window: &MainWindow, message: impl Into<slint::SharedString>) {
	show(window, message, NotificationKind::Info);
}

pub fn error_async(window: &slint::Weak<MainWindow>, message: impl Into<String>) {
	show_async(window, message, NotificationKind::Error);
}

fn show_async(
	window: &slint::Weak<MainWindow>,
	message: impl Into<String>,
	kind: NotificationKind,
) {
	let window = window.clone();
	let message = message.into();

	let _ = slint::invoke_from_event_loop(move || {
		let Some(window) = window.upgrade() else {
			return;
		};

		show(&window, message, kind);
	});
}

pub fn dismiss(window: &MainWindow, id: i32) {
	let notifications = window
		.get_notifications()
		.iter()
		.filter(|notification| notification.id != id)
		.collect::<Vec<_>>();

	window.set_notifications(Rc::new(slint::VecModel::from(notifications)).into());
}

fn show(window: &MainWindow, message: impl Into<slint::SharedString>, kind: NotificationKind) {
	let mut notifications = window.get_notifications().iter().collect::<Vec<_>>();

	notifications.push(NotificationData {
		id: NEXT_ID.fetch_add(1, Ordering::Relaxed),
		message: message.into(),
		kind,
	});

	window.set_notifications(Rc::new(slint::VecModel::from(notifications)).into());
}
