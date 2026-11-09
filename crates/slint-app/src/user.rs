use slint::ComponentHandle as _;
use std::{path::Path, thread};

use rai_pal_core::{
	result::LogErrExt,
	user::auth::{AuthState, get_user_auth_state, logout_auth, start_auth},
};

use crate::{MainWindow, analytics, notifications, runtime, text};

pub fn load(window: slint::Weak<MainWindow>) {
	refresh_state(window);
}

pub fn log_in(window: slint::Weak<MainWindow>) {
	let Some(handle) = window.upgrade() else {
		return;
	};

	if handle.global::<crate::UserState>().get_state() == crate::ActionState::Loading {
		return;
	}

	handle
		.global::<crate::UserState>()
		.set_state(crate::ActionState::Loading);
	drop(handle);

	thread::spawn(move || {
		let result = runtime::block_on(async {
			start_auth().await?;
			Ok(get_user_auth_state().await?)
		});

		if result.is_ok() {
			analytics::send(rai_pal_core::analytics::Event::UserSignIn, &[]);
		}

		apply_result(window, result);
	});
}

pub fn log_out(window: slint::Weak<MainWindow>) {
	let Some(handle) = window.upgrade() else {
		return;
	};

	if handle.global::<crate::UserState>().get_state() == crate::ActionState::Loading {
		return;
	}

	handle
		.global::<crate::UserState>()
		.set_state(crate::ActionState::Loading);
	drop(handle);

	thread::spawn(move || {
		let result = runtime::block_on(async {
			logout_auth().await?;
			Ok(get_user_auth_state().await?)
		});

		apply_result(window, result);
	});
}

fn refresh_state(window: slint::Weak<MainWindow>) {
	thread::spawn(move || {
		let result = runtime::block_on(get_user_auth_state());

		apply_result(window, result);
	});
}

fn apply_result(window: slint::Weak<MainWindow>, result: Result<AuthState, String>) {
	crate::runtime::invoke_on_ui(move || {
		let Some(window) = window.upgrade() else {
			return;
		};

		window
			.global::<crate::UserState>()
			.set_state(crate::ActionState::Idle);

		match result {
			Ok(state) => apply_state(&window, &state),
			Err(err) => {
				log::error!("Failed to refresh user auth state: {err}");
				notifications::error(&window, format!("Failed to refresh user auth state: {err}"));
			}
		}
	});
}

fn apply_state(window: &MainWindow, state: &AuthState) {
	let avatar = state
		.avatar_path
		.as_deref()
		.map(Path::new)
		.and_then(|path| {
			slint::Image::load_from_path(path).ok_or_log("Failed to load user avatar")
		});

	window
		.global::<crate::UserState>()
		.set_logged_in(state.is_logged_in);
	window
		.global::<crate::UserState>()
		.set_name(state.user_name.clone().unwrap_or_default().into());
	window.global::<crate::UserState>().set_initials(
		text::initials(state.user_name.as_deref().unwrap_or_default())
			.unwrap_or_default()
			.into(),
	);
	window
		.global::<crate::UserState>()
		.set_has_avatar(avatar.is_some());
	window
		.global::<crate::UserState>()
		.set_avatar(avatar.unwrap_or_default());
}
