use std::{sync::Arc, thread, time::Duration};

use rai_pal_backend::{
	games_query::GamesQuery,
	local_database::{
		app_database::DbMutex,
		game_database::{GameDatabase, GameIdsResponse},
	},
};
use slint::ComponentHandle as _;

use crate::{
	MainWindow, MessageContext,
	model::make_rows,
	notifications,
	scheduler::FetchScheduler,
	scheduler::{enqueue_index, schedule_visible_range},
};

use super::{CommandHandler, on_command};

#[derive(Clone, Copy)]
pub enum ScrollBehavior {
	Reset,
	Keep,
}

pub fn load_game_ids(database: &DbMutex, query: GamesQuery) -> Result<GameIdsResponse, String> {
	database
		.get_game_ids(Some(query))
		.map_err(|err| err.to_string())
}

impl CommandHandler {
	pub fn start_query(&self, delay: Duration, scroll: ScrollBehavior) {
		run_query(
			self.window(),
			Arc::clone(&self.database),
			Arc::clone(&self.scheduler),
			self.query(),
			delay,
			scroll,
		);
	}

	pub fn enqueue_game(&self, index: i32) {
		let Some(window) = self.ui() else {
			return;
		};

		let rows = window.global::<crate::GamesList>().get_rows();
		enqueue_index(
			&self.scheduler,
			&rows,
			usize::try_from(index.max(0)).unwrap_or_default(),
		);
	}

	pub fn schedule_visible_range(&self, first: i32, last: i32) {
		if last < 0 {
			return;
		}

		let Some(window) = self.ui() else {
			return;
		};

		let rows = window.global::<crate::GamesList>().get_rows();
		schedule_visible_range(
			&self.scheduler,
			&rows,
			usize::try_from(first.max(0)).unwrap_or_default(),
			usize::try_from(last).unwrap_or_default(),
		);
	}
}

fn run_query(
	window: slint::Weak<MainWindow>,
	database: Arc<DbMutex>,
	scheduler: Arc<FetchScheduler>,
	query: GamesQuery,
	delay: Duration,
	scroll: ScrollBehavior,
) {
	let query_generation = scheduler.begin_query();

	thread::spawn(move || {
		if !delay.is_zero() {
			thread::sleep(delay);
		}

		if scheduler.generation() != query_generation {
			return;
		}

		let result = load_game_ids(&database, query);
		let ui_scheduler = Arc::clone(&scheduler);

		crate::runtime::invoke_on_ui(move || {
			if ui_scheduler.generation() != query_generation {
				return;
			}

			let Some(window) = window.upgrade() else {
				return;
			};

			let response = match result {
				Ok(response) => response,
				Err(err) => {
					log::error!("Failed to get app data: {err}");
					notifications::error(
						&window,
						notifications::data(MessageContext::LoadAppData, &err),
					);
					return;
				}
			};

			let rows = make_rows(&response.game_ids);
			ui_scheduler.begin_query();

			let list = window.global::<crate::GamesList>();
			let scroll_y = match scroll {
				ScrollBehavior::Reset => 0.0,
				ScrollBehavior::Keep => list.get_scroll_y(),
			};

			window.global::<crate::GamesList>().set_total_count(
				i32::try_from(response.total_count).unwrap_or_else(|err| {
					log::warn!("Games count exceeds i32: {err}");
					i32::MAX
				}),
			);
			let model = std::rc::Rc::new(slint::VecModel::from(rows));
			list.set_rows(model.into());
			list.set_scroll_y(scroll_y);

			let window = window.as_weak();

			// Replacing the model makes the ListView snap content-y to a row boundary,
			// so the scroll position has to be restored after the next layout pass.
			slint::Timer::single_shot(Duration::from_millis(16), move || {
				if let Some(window) = window.upgrade() {
					window.global::<crate::GamesList>().set_scroll_y(scroll_y);
				}
			});
		});
	});
}

pub fn register(window: &MainWindow) {
	on_command!(window, GamesActions, on_request_game, |index| handler => {
		handler.enqueue_game(index);
	});
	on_command!(window, GamesActions, on_visible_range_changed, |first, last| handler => {
		handler.schedule_visible_range(first, last);
	});
}
