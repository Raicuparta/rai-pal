use std::{
	future::Future,
	sync::{
		LazyLock,
		atomic::{AtomicBool, Ordering},
	},
	time::Duration,
};

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
	run(future).map_err(|err| err.to_string())
}

pub fn run<T>(future: impl Future<Output = T>) -> T {
	RUNTIME.block_on(future)
}

pub fn spawn(future: impl Future<Output = ()> + Send + 'static) {
	RUNTIME.spawn(future);
}

static POINTER_PRESSED: AtomicBool = AtomicBool::new(false);

pub fn set_pointer_pressed(pressed: bool) {
	POINTER_PRESSED.store(pressed, Ordering::Relaxed);
}

pub fn invoke_on_ui(callback: impl FnOnce() + Send + 'static) {
	if let Err(err) = slint::invoke_from_event_loop(move || {
		run_when_pointer_released(Box::new(callback));
	}) {
		log::warn!("Failed to invoke UI callback: {err}");
	}
}

fn run_when_pointer_released(callback: Box<dyn FnOnce()>) {
	if POINTER_PRESSED.load(Ordering::Relaxed) {
		slint::Timer::single_shot(Duration::from_millis(50), move || {
			run_when_pointer_released(callback);
		});
	} else {
		callback();
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
