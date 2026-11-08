use std::{
	rc::Rc,
	sync::atomic::{AtomicI32, Ordering},
	thread,
	time::Duration,
};

use slint::{ComponentHandle, Model};

use crate::{MainWindow, NotificationKind, NotificationView, analytics};

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

	crate::runtime::invoke_on_ui(move || {
		let Some(window) = window.upgrade() else {
			return;
		};

		show(&window, message, kind);
	});
}

pub fn dismiss(window: &MainWindow, id: i32) {
	let notifications = window.get_notifications();

	if !notifications
		.iter()
		.any(|notification| notification.id == id)
	{
		return;
	}

	let notifications = notifications
		.iter()
		.filter(|notification| notification.id != id)
		.collect::<Vec<_>>();

	window.set_notifications(Rc::new(slint::VecModel::from(notifications)).into());
}

fn schedule_dismiss(window: slint::Weak<MainWindow>, id: i32) {
	thread::spawn(move || {
		thread::sleep(Duration::from_secs(5));

		crate::runtime::invoke_on_ui(move || {
			if let Some(window) = window.upgrade() {
				dismiss(&window, id);
			}
		});
	});
}

fn show(window: &MainWindow, message: impl Into<slint::SharedString>, kind: NotificationKind) {
	let mut notifications = window.get_notifications().iter().collect::<Vec<_>>();
	let message = message.into();

	if kind == NotificationKind::Error {
		analytics::send(
			rai_pal_core::analytics::Event::ErrorNotification,
			&[("error_message", message.to_string())],
		);
	}

	let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
	notifications.push(NotificationView { id, message, kind });

	window.set_notifications(Rc::new(slint::VecModel::from(notifications)).into());

	schedule_dismiss(window.as_weak(), id);
}
