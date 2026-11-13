use std::future::Future;

pub fn current_thread() -> std::io::Result<tokio::runtime::Runtime> {
	tokio::runtime::Builder::new_current_thread()
		.enable_all()
		.build()
}

pub fn block_on<T>(
	future: impl Future<Output = rai_pal_backend::result::Result<T>>,
) -> Result<T, String> {
	current_thread()
		.map_err(|err| err.to_string())?
		.block_on(future)
		.map_err(|err| err.to_string())
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
