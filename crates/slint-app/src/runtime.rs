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
