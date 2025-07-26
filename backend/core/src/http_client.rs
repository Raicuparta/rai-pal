use std::sync::OnceLock;

use reqwest::{Client, redirect::Policy};

static HTTP_CLIENT: OnceLock<Client> = OnceLock::new();
static HEAD_CLIENT: OnceLock<Client> = OnceLock::new();

pub fn get_client() -> &'static Client {
	HTTP_CLIENT.get_or_init(|| {
		Client::builder()
			.build()
			.expect("Failed to create HTTP client")
	})
}

pub fn get_client_no_redirect() -> &'static Client {
	HEAD_CLIENT.get_or_init(|| {
		Client::builder()
			.redirect(Policy::none())
			.build()
			.expect("Failed to create HEAD HTTP client")
	})
}
