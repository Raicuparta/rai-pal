use std::{path::Path, thread};

use rai_pal_core::user::auth::{AuthState, get_user_auth_state, logout_auth, start_auth};

use crate::MainWindow;

pub fn load(window: slint::Weak<MainWindow>) {
	refresh_state(window);
}

pub fn log_in(window: slint::Weak<MainWindow>) {
	let Some(handle) = window.upgrade() else {
		return;
	};

	if handle.get_user_busy() {
		return;
	}

	handle.set_user_busy(true);
	drop(handle);

	thread::spawn(move || {
		let result = run_async(async {
			start_auth().await?;
			Ok(get_user_auth_state().await?)
		});

		apply_result(window, result);
	});
}

pub fn log_out(window: slint::Weak<MainWindow>) {
	let Some(handle) = window.upgrade() else {
		return;
	};

	if handle.get_user_busy() {
		return;
	}

	handle.set_user_busy(true);
	drop(handle);

	thread::spawn(move || {
		let result = run_async(async {
			logout_auth().await?;
			Ok(get_user_auth_state().await?)
		});

		apply_result(window, result);
	});
}

fn refresh_state(window: slint::Weak<MainWindow>) {
	thread::spawn(move || {
		let result = run_async(get_user_auth_state());

		apply_result(window, result);
	});
}

fn apply_result(window: slint::Weak<MainWindow>, result: Result<AuthState, String>) {
	let _ = slint::invoke_from_event_loop(move || {
		let Some(window) = window.upgrade() else {
			return;
		};

		window.set_user_busy(false);

		match result {
			Ok(state) => apply_state(&window, &state),
			Err(err) => eprintln!("Failed to refresh user auth state: {err}"),
		}
	});
}

fn apply_state(window: &MainWindow, state: &AuthState) {
	let avatar = state
		.avatar_path
		.as_deref()
		.map(Path::new)
		.and_then(|path| slint::Image::load_from_path(path).ok());

	window.set_user_logged_in(state.is_logged_in);
	window.set_user_name(state.user_name.clone().unwrap_or_default().into());
	window.set_user_initials(
		get_initials(state.user_name.as_deref())
			.unwrap_or_default()
			.into(),
	);
	window.set_user_has_avatar(avatar.is_some());
	window.set_user_avatar(avatar.unwrap_or_default());
}

fn run_async<T>(
	future: impl std::future::Future<Output = rai_pal_core::result::Result<T>>,
) -> Result<T, String> {
	let runtime = tokio::runtime::Builder::new_current_thread()
		.enable_all()
		.build()
		.map_err(|err| err.to_string())?;

	runtime.block_on(future).map_err(|err| err.to_string())
}

fn get_initials(name: Option<&str>) -> Option<String> {
	let words = name?.split_whitespace().collect::<Vec<_>>();

	if words.is_empty() {
		return None;
	}

	let initials = if words.len() == 1 {
		words[0]
			.chars()
			.filter(|character| character.is_ascii_alphanumeric())
			.collect::<String>()
	} else {
		words
			.iter()
			.filter_map(|word| {
				word.chars()
					.find(|character| character.is_ascii_alphanumeric())
			})
			.collect::<String>()
	};

	let initials = initials.chars().take(2).collect::<String>().to_uppercase();

	if initials.is_empty() {
		None
	} else {
		Some(initials)
	}
}
