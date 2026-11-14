use std::{future::Future, sync::LazyLock};

static RUNTIME: LazyLock<tokio::runtime::Runtime> = LazyLock::new(|| {
	#[expect(
		clippy::expect_used,
		reason = "Startup invariant — the app cannot function without an async runtime"
	)]
	tokio::runtime::Builder::new_multi_thread()
		.enable_all()
		.build()
		.expect("Failed to set up async runtime")
});

pub fn block_on<T>(
	future: impl Future<Output = rai_pal_backend::result::Result<T>>,
) -> Result<T, String> {
	RUNTIME.block_on(future).map_err(|err| err.to_string())
}

pub fn spawn(future: impl Future<Output = ()> + Send + 'static) {
	let _ = RUNTIME.spawn(future);
}

pub fn invoke_on_ui(callback: impl FnOnce() + Send + 'static) {
	if let Err(err) = slint::invoke_from_event_loop(callback) {
		log::warn!("Failed to invoke UI callback: {err}");
	}
}

pub fn with_ui_window(
	window: &slint::Weak<crate::MainWindow>,
	callback: impl FnOnce(crate::MainWindow) + Send + 'static,
) {
	let window = window.clone();

	invoke_on_ui(move || {
		if let Some(window) = window.upgrade() {
			callback(window);
		}
	});
}
