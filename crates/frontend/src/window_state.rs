use rai_pal_backend::app_settings::{AppSettings, WindowState};
use slint::{
	ComponentHandle as _,
	winit_030::{EventResult, WinitWindowAccessor, winit},
};

use crate::{MainWindow, commands::CommandHandler};

pub fn apply(window: &MainWindow, state: Option<WindowState>) {
	let Some(state) = state else {
		return;
	};

	// Gamescope (Steam Deck game mode) scales windows larger than its output for display,
	// but pointer input stays in unscaled window coordinates, leaving the bottom/right part
	// of the window unreachable. Never restore a size that doesn't fit the current monitor.
	let monitor_size = window
		.window()
		.with_winit_window(|winit_window| {
			winit_window
				.current_monitor()
				.or_else(|| winit_window.primary_monitor())
				.map(|monitor| monitor.size())
		})
		.flatten();

	let (width, height) = monitor_size.map_or((state.width, state.height), |monitor_size| {
		(
			state.width.min(monitor_size.width),
			state.height.min(monitor_size.height),
		)
	});

	if (width, height) != (state.width, state.height) {
		log::warn!(
			"Clamping restored window size {}x{} to {}x{}",
			state.width,
			state.height,
			width,
			height
		);
	}

	if width > 0 && height > 0 {
		window
			.window()
			.set_size(slint::PhysicalSize::new(width, height));
	}

	let position = monitor_size.map_or_else(
		|| slint::PhysicalPosition::new(state.x, state.y),
		|monitor_size| {
			slint::PhysicalPosition::new(
				state.x.clamp(
					0,
					i32::try_from(monitor_size.width.saturating_sub(width)).unwrap_or(i32::MAX),
				),
				state.y.clamp(
					0,
					i32::try_from(monitor_size.height.saturating_sub(height)).unwrap_or(i32::MAX),
				),
			)
		},
	);

	window.window().set_position(position);

	if state.maximized {
		window.window().set_maximized(true);
	}
}

pub fn register_events(window: &MainWindow) {
	#[cfg(target_os = "windows")]
	let mut window_icon_applied = false;

	window.window().on_winit_window_event(move |window, event| {
		#[cfg(target_os = "windows")]
		{
			let icon_needs_update = match event {
				winit::event::WindowEvent::Resized(_)
				| winit::event::WindowEvent::RedrawRequested => !window_icon_applied,
				winit::event::WindowEvent::ScaleFactorChanged { .. } => window_icon_applied,
				_ => false,
			};
			if icon_needs_update && apply_windows_window_icon(window) {
				window_icon_applied = true;
			}
		}

		match event {
			winit::event::WindowEvent::Resized(_) | winit::event::WindowEvent::Moved(_) => {
				if let Some(handler) = CommandHandler::current() {
					update_state(window, &mut handler.settings.borrow_mut());
				}
			}
			winit::event::WindowEvent::CloseRequested | winit::event::WindowEvent::Destroyed => {
				if let Some(handler) = CommandHandler::current() {
					update_state(window, &mut handler.settings.borrow_mut());
					handler.persist_settings();
				}
			}
			winit::event::WindowEvent::Focused(true) => {
				if let Some(handler) = CommandHandler::current() {
					handler.refresh_selected_game_on_focus();
				}
			}
			winit::event::WindowEvent::HoveredFile(_) => {
				if let Some(handler) = CommandHandler::current()
					&& let Some(app_window) = handler.ui()
				{
					app_window.set_drop_highlight(true);
				}
			}
			winit::event::WindowEvent::HoveredFileCancelled => {
				if let Some(handler) = CommandHandler::current()
					&& let Some(app_window) = handler.ui()
				{
					app_window.set_drop_highlight(false);
				}
			}
			winit::event::WindowEvent::DroppedFile(path) => {
				if let Some(handler) = CommandHandler::current() {
					if let Some(app_window) = handler.ui() {
						app_window.set_drop_highlight(false);
					}
					handler.add_dropped_paths(vec![path.clone()]);
				}
			}
			_ => {}
		}

		EventResult::Propagate
	});
}

#[cfg(target_os = "windows")]
fn apply_windows_window_icon(window: &slint::Window) -> bool {
	use image::imageops::FilterType;
	use slint::winit_030::winit::{platform::windows::WindowExtWindows, window::Icon};

	let Ok(source) = image::load_from_memory(include_bytes!("../graphics/128x128.png")) else {
		return false;
	};
	let source = source.to_rgba8();

	window
		.with_winit_window(|winit_window| {
			let scale_factor = f64::from(winit_window.scale_factor());
			let make_icon = |logical_size: f64| {
				let size = (logical_size * scale_factor).round().clamp(1.0, 128.0) as u32;
				let pixels = image::imageops::resize(&source, size, size, FilterType::Lanczos3);
				Icon::from_rgba(pixels.into_raw(), size, size).ok()
			};

			if let Some(icon) = make_icon(16.0) {
				winit_window.set_window_icon(Some(icon));
			}
			if let Some(icon) = make_icon(32.0) {
				winit_window.set_taskbar_icon(Some(icon));
			}
		})
		.is_some()
}

fn update_state(window: &slint::Window, settings: &mut AppSettings) {
	let maximized = window.is_maximized();
	let position = window.position();
	let size = window.size();

	let state = settings.window_state.get_or_insert(WindowState {
		x: position.x,
		y: position.y,
		width: size.width,
		height: size.height,
		maximized,
	});

	state.maximized = maximized;
	if !maximized && size.width > 0 && size.height > 0 {
		state.x = position.x;
		state.y = position.y;
		state.width = size.width;
		state.height = size.height;
	}
}

/// Gamescope's Xwayland reports bogus output dimensions and no `Xft.dpi`, so winit's X11
/// DPI heuristic derives a ~2.17x scale factor from them. Gamescope already scales the
/// window to the display, so the UI must be rendered at 1x there.
pub fn fix_gamescope_scale_factor() {
	let in_gamescope = std::env::var_os("GAMESCOPE_WAYLAND_DISPLAY").is_some()
		|| std::env::var("XDG_SESSION_DESKTOP").is_ok_and(|value| value == "gamescope");

	if !in_gamescope
		|| std::env::var_os("SLINT_SCALE_FACTOR").is_some()
		|| std::env::var_os("WINIT_X11_SCALE_FACTOR").is_some()
	{
		return;
	}

	// SAFETY: called before the Slint backend is initialized and before any threads are spawned.
	unsafe {
		std::env::set_var("SLINT_SCALE_FACTOR", "1");
	}
}

pub fn platform_name() -> String {
	#[cfg(target_os = "linux")]
	{
		if std::env::var("WAYLAND_DISPLAY").is_ok_and(|value| !value.is_empty()) {
			return "Linux (Wayland)".to_string();
		}

		if std::env::var("DISPLAY").is_ok_and(|value| !value.is_empty()) {
			return "Linux (X11)".to_string();
		}

		"Linux".to_string()
	}

	#[cfg(target_os = "windows")]
	{
		"Windows".to_string()
	}

	#[cfg(not(any(target_os = "linux", target_os = "windows")))]
	{
		std::env::consts::OS.to_string()
	}
}
