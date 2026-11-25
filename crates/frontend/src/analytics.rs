use std::{
	collections::VecDeque,
	sync::{LazyLock, Once, mpsc},
	thread,
};

use rai_pal_backend::analytics::{self as core, Event};

use crate::runtime;

static STARTED: Once = Once::new();

static SINK: LazyLock<mpsc::Sender<Event>> = LazyLock::new(|| {
	let (sender, receiver) = mpsc::channel();
	thread::spawn(move || collect_events(&receiver));
	sender
});

pub fn start_app() {
	STARTED.call_once(|| {
		record(Event::StartApp {
			app_version: env!("CARGO_PKG_VERSION").to_string(),
		});
	});
}

pub fn record(event: Event) {
	record_all([event]);
}

pub fn record_all(events: impl IntoIterator<Item = Event>) {
	for event in events {
		if SINK.send(event).is_err() {
			log::warn!("Analytics sink stopped unexpectedly");
			return;
		}
	}
}

fn collect_events(receiver: &mpsc::Receiver<Event>) {
	let session = core::Session::new();
	let mut queue = VecDeque::new();

	loop {
		let Ok(event) = receiver.recv() else {
			return;
		};

		queue.push_back(event);

		while let Ok(queued) = receiver.try_recv() {
			queue.push_back(queued);
		}

		flush(&session, &mut queue);
	}
}

fn flush(session: &core::Session, queue: &mut VecDeque<Event>) {
	while !queue.is_empty() {
		let count = queue.len().min(core::MAX_EVENTS_PER_REQUEST);
		let batch = queue.iter().take(count).cloned().collect();

		if let Err(err) = runtime::block_on(session.send(batch)) {
			log::warn!("Failed to send analytics events: {err}");
			return;
		}

		queue.drain(..count);
	}
}
