use std::{
	fmt::Display,
	rc::Rc,
	sync::atomic::{AtomicI32, Ordering},
	time::Duration,
};

use slint::{ComponentHandle, Model, SharedString};

use crate::{
	MainWindow, MessageContext, MessageData, NotificationData, NotificationKind, analytics,
};

static NEXT_ID: AtomicI32 = AtomicI32::new(1);

pub fn data(context: MessageContext, detail: impl Display) -> MessageData {
	MessageData {
		context,
		subject: SharedString::default(),
		detail: detail.to_string().into(),
	}
}

pub fn data_with_subject(
	context: MessageContext,
	subject: impl Into<SharedString>,
	detail: impl Display,
) -> MessageData {
	MessageData {
		context,
		subject: subject.into(),
		detail: detail.to_string().into(),
	}
}

pub fn error(window: &MainWindow, message: MessageData) {
	show(window, message, NotificationKind::Error);
}

pub fn info(window: &MainWindow, message: MessageData) {
	show(window, message, NotificationKind::Info);
}

pub fn error_async(window: &slint::Weak<MainWindow>, message: MessageData) {
	show_async(window, message, NotificationKind::Error);
}

fn show_async(window: &slint::Weak<MainWindow>, message: MessageData, kind: NotificationKind) {
	let window = window.clone();

	crate::runtime::invoke_on_ui(move || {
		let Some(window) = window.upgrade() else {
			return;
		};

		show(&window, message, kind);
	});
}

pub fn dismiss(window: &MainWindow, id: i32) {
	let notifications = window
		.global::<crate::NotificationsState>()
		.get_notifications();

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

	window
		.global::<crate::NotificationsState>()
		.set_notifications(Rc::new(slint::VecModel::from(notifications)).into());
}

fn schedule_dismiss(window: slint::Weak<MainWindow>, id: i32) {
	slint::Timer::single_shot(Duration::from_secs(5), move || {
		if let Some(window) = window.upgrade() {
			dismiss(&window, id);
		}
	});
}

fn show(window: &MainWindow, message: MessageData, kind: NotificationKind) {
	let mut notifications = window
		.global::<crate::NotificationsState>()
		.get_notifications()
		.iter()
		.collect::<Vec<_>>();

	if kind == NotificationKind::Error {
		analytics::send(
			rai_pal_backend::analytics::Event::ErrorNotification,
			[("error_message", message.detail.to_string())],
		);
	}

	let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);

	notifications.push(NotificationData { id, message, kind });

	window
		.global::<crate::NotificationsState>()
		.set_notifications(Rc::new(slint::VecModel::from(notifications)).into());

	schedule_dismiss(window.as_weak(), id);
}

pub fn register(window: &MainWindow) {
	let ui_window = window.as_weak();
	window
		.global::<crate::AppActions>()
		.on_dismiss_notification(move |id| {
			if let Some(window) = ui_window.upgrade() {
				dismiss(&window, id);
			}
		});
}
