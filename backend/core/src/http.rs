use std::{sync::LazyLock, time::Duration};

pub static CLIENT: LazyLock<reqwest::Client> = LazyLock::new(|| {
	reqwest::Client::builder()
		.timeout(Duration::from_secs(10))
		.pool_max_idle_per_host(10)
		.build()
		.expect("NOOOO")
});

pub static CLIENT_NO_REDIRECT: LazyLock<reqwest::Client> = LazyLock::new(|| {
	reqwest::Client::builder()
		.redirect(reqwest::redirect::Policy::none())
		.timeout(Duration::from_secs(10))
		.pool_max_idle_per_host(10)
		.build()
		.expect("NOOOO")
});
