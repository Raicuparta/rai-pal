use std::{
	cell::RefCell,
	path::PathBuf,
	sync::{
		Arc,
		atomic::{AtomicU64, Ordering},
		mpsc,
	},
	thread,
	time::Duration,
};

use notify::{EventKind, RecursiveMode, Watcher};
use rai_pal_backend::{open_better::open_detached_better, path_extensions::PathExt};
use slint::ComponentHandle;

use crate::{ActionKind, MainWindow, MessageContext, TextEditorState, action_state, notifications};

use super::{CommandHandler, on_command};

#[derive(Default)]
pub struct TextEditor {
	open: RefCell<Option<OpenTextEditor>>,
	generation: Arc<AtomicU64>,
}

struct OpenTextEditor {
	target: TextEditorTarget,
	_watcher: Option<notify::RecommendedWatcher>,
}

#[derive(Clone)]
struct TextEditorTarget {
	path: PathBuf,
}

impl TextEditor {
	fn open(&self, path: PathBuf, content: String) {
		let watch_id = self.generation.fetch_add(1, Ordering::Relaxed) + 1;
		self.open.borrow_mut().take();

		let watcher = watch_editor_file(
			path.clone(),
			content,
			Arc::clone(&self.generation),
			watch_id,
		);

		self.open.replace(Some(OpenTextEditor {
			target: TextEditorTarget { path },
			_watcher: watcher,
		}));
	}

	fn target(&self) -> Option<TextEditorTarget> {
		self.open.borrow().as_ref().map(|open| open.target.clone())
	}

	fn close(&self) {
		self.generation.fetch_add(1, Ordering::Relaxed);
		self.open.borrow_mut().take();
	}
}

impl CommandHandler {
	pub fn open_text_editor(&self, path: PathBuf, text: String, title: String) {
		self.open_text_file(path, text, title, false);
	}

	pub fn view_text_file(&self, path: PathBuf, text: String, title: String) {
		self.open_text_file(path, text, title, true);
	}

	fn open_text_file(&self, path: PathBuf, text: String, title: String, read_only: bool) {
		let Some(window) = self.ui() else {
			return;
		};

		let state = window.global::<TextEditorState>();
		state.set_title(title.into());
		state.set_dirty(false);
		state.set_read_only(read_only);
		state.set_text(text.clone().into());
		state.set_reset_scroll_revision(state.get_reset_scroll_revision() + 1);
		state.set_open(true);

		self.text_editor.open(path, text);
	}

	pub fn save_text_editor(&self) {
		let Some(target) = self.text_editor.target() else {
			return;
		};
		let Some(window) = self.ui() else {
			return;
		};
		let content = window.global::<TextEditorState>().get_text();
		let action_id = action_state::key(
			ActionKind::SaveTextEditor,
			window.global::<TextEditorState>().get_title(),
		);
		let dirty_window = window.as_weak();

		self.spawn_action(action_id, MessageContext::None, move || {
			fs_err::write(&target.path, content.as_str())?;

			crate::runtime::invoke_on_ui(move || {
				if let Some(upgraded_window) = dirty_window.upgrade() {
					upgraded_window.global::<TextEditorState>().set_dirty(false);
				}
			});

			Ok::<(), std::io::Error>(())
		});
	}

	pub fn open_text_editor_path(&self, open_folder: bool) {
		let Some(target) = self.text_editor.target() else {
			return;
		};
		let Some(window) = self.ui() else {
			return;
		};

		let result = if open_folder || !target.path.is_file() {
			target.path.open_folder_or_parent()
		} else {
			open_detached_better(&target.path)
		};

		if let Err(err) = result {
			notifications::error(&window, notifications::data(MessageContext::None, &err));
		}
	}

	pub fn close_text_editor(&self) {
		self.text_editor.close();
	}

	pub fn mark_text_editor_dirty(&self) {
		let Some(window) = self.ui() else {
			return;
		};

		window.global::<TextEditorState>().set_dirty(true);
	}

	fn apply_external_text_editor_content(&self, text: &str) {
		let Some(window) = self.ui() else {
			return;
		};

		let state = window.global::<TextEditorState>();
		state.set_scroll_revision(state.get_scroll_revision() + 1);
		state.set_text(text.into());
	}
}

fn watch_editor_file(
	path: PathBuf,
	mut last_content: String,
	generation: Arc<AtomicU64>,
	watch_id: u64,
) -> Option<notify::RecommendedWatcher> {
	let (sender, receiver) = mpsc::channel();
	let mut watcher = match notify::recommended_watcher(move |event| {
		let _ = sender.send(event);
	}) {
		Ok(watcher) => watcher,
		Err(err) => {
			log::warn!("Failed to create editor file watcher: {err}");
			return None;
		}
	};

	let parent = path.parent()?;

	if let Err(err) = watcher.watch(parent, RecursiveMode::NonRecursive) {
		log::warn!("Failed to watch editor file: {err}");
		return None;
	}

	thread::spawn(move || {
		loop {
			match receiver.recv() {
				Ok(Ok(event)) => {
					if matches!(event.kind, EventKind::Access(_)) {
						continue;
					}
				}
				Ok(Err(err)) => {
					log::warn!("Editor file watcher error: {err}");
					continue;
				}
				Err(_) => return,
			}

			if generation.load(Ordering::Relaxed) != watch_id {
				return;
			}

			while receiver.recv_timeout(Duration::from_millis(50)).is_ok() {}

			if generation.load(Ordering::Relaxed) != watch_id {
				return;
			}

			let Ok(content) = fs_err::read_to_string(&path) else {
				continue;
			};

			if content == last_content {
				continue;
			}
			last_content.clone_from(&content);

			let result = slint::invoke_from_event_loop({
				let generation = Arc::clone(&generation);
				move || {
					if generation.load(Ordering::Relaxed) != watch_id {
						return;
					}
					let Some(handler) = CommandHandler::current() else {
						return;
					};
					handler.apply_external_text_editor_content(&content);
				}
			});

			if result.is_err() {
				return;
			}
		}
	});

	Some(watcher)
}

pub fn register(window: &MainWindow) {
	on_command!(window, TextEditorState, on_request_save, || handler => {
		handler.save_text_editor();
	});
	on_command!(window, TextEditorState, on_text_changed, |_text| handler => {
		handler.mark_text_editor_dirty();
	});
	on_command!(window, TextEditorState, on_request_open_file, || handler => {
		handler.open_text_editor_path(false);
	});
	on_command!(window, TextEditorState, on_request_open_folder, || handler => {
		handler.open_text_editor_path(true);
	});
	on_command!(window, TextEditorState, on_closed, || handler => {
		handler.close_text_editor();
	});
}
