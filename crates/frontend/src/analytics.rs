use std::collections::HashMap;

use rai_pal_backend::{analytics as core, game::Game};

use crate::runtime;

pub fn send(event: core::Event, data: impl IntoIterator<Item = (&'static str, String)>) {
	let data = data
		.into_iter()
		.map(|(key, value)| (key.to_string(), value))
		.collect::<HashMap<_, _>>();

	runtime::spawn(async move {
		core::send_event(event, Some(data)).await;
	});
}

pub fn send_event(event: core::Event) {
	send(event, std::iter::empty());
}

pub fn send_mod(event: core::Event, mod_id: &str, game: Option<&Game>) {
	let mut data = vec![("mod_id", mod_id.to_string())];

	if let Some(game) = game {
		data.push(("game", game.display_title.clone()));
	}

	send(event, data);
}
