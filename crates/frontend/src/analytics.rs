use std::{collections::HashMap, thread};

use rai_pal_backend::{analytics as core, game::Game};

use rai_pal_backend::result::LogErrExt;

use crate::runtime;

pub fn send(event: core::Event, data: &[(&str, String)]) {
	let data = data
		.iter()
		.map(|(key, value)| ((*key).to_string(), value.clone()))
		.collect::<HashMap<_, _>>();

	thread::spawn(move || {
		let Some(runtime) = runtime::current_thread().ok_or_log("Failed to create async runtime")
		else {
			return;
		};

		runtime.block_on(core::send_event(event, Some(data)));
	});
}

pub fn send_mod(event: core::Event, mod_id: &str, game: Option<&Game>) {
	let mut data = vec![("mod_id".to_string(), mod_id.to_string())];
	if let Some(game) = game {
		data.push(("game".to_string(), game.display_title.clone()));
	}

	send(
		event,
		&data
			.iter()
			.map(|(key, value)| (key.as_str(), value.clone()))
			.collect::<Vec<_>>(),
	);
}
