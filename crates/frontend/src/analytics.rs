use std::{
	sync::{LazyLock, mpsc},
	thread,
};

use rai_pal_backend::analytics::{self as core, Event};

use crate::runtime;

static SINK: LazyLock<mpsc::Sender<Event>> = LazyLock::new(|| {
	let (sender, receiver) = mpsc::channel();
	thread::spawn(move || collect_events(receiver));
	sender
});

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

fn collect_events(receiver: mpsc::Receiver<Event>) {
	let mut batch = Vec::new();

	loop {
		let Ok(event) = receiver.recv() else {
			return;
		};

		batch.push(event);

		while batch.len() < core::MAX_EVENTS_PER_REQUEST {
			match receiver.try_recv() {
				Ok(event) => batch.push(event),
				Err(_) => break,
			}
		}

		flush(&mut batch);
	}
}

fn flush(batch: &mut Vec<Event>) {
	if batch.is_empty() {
		return;
	}

	if let Err(err) = runtime::block_on(core::send_batch(std::mem::take(batch))) {
		log::warn!("Failed to send analytics events: {err}");
	}
}
