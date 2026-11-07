use std::future::Future;

pub fn current_thread() -> std::io::Result<tokio::runtime::Runtime> {
	tokio::runtime::Builder::new_current_thread()
		.enable_all()
		.build()
}

pub fn block_on<T>(
	future: impl Future<Output = rai_pal_core::result::Result<T>>,
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
