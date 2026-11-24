use rai_pal_backend::local_database::mod_database::ModGame;

use crate::{ActionKey, ActionKind, ModActionData, ModActionKind, ProviderId};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ActionGroup {
	Path,
	Primary,
	Manage,
	Link,
	Games,
	Debug,
}

pub struct ModActionList {
	mod_id: String,
	group: Option<ActionGroup>,
	actions: Vec<ModActionData>,
}

impl ModActionList {
	pub fn new(mod_id: &str) -> Self {
		Self {
			mod_id: mod_id.to_string(),
			group: None,
			actions: Vec::new(),
		}
	}

	pub fn action(
		&self,
		kind: ModActionKind,
		action_kind: ActionKind,
		target: &str,
	) -> ModActionData {
		ModActionData {
			kind,
			action_key: ActionKey {
				kind: action_kind,
				target: self.mod_id.as_str().into(),
			},
			mod_id: self.mod_id.as_str().into(),
			subject: "".into(),
			target: target.into(),
			provider: ProviderId::None,
			enabled: true,
			separator_before: false,
			open_folder: false,
		}
	}

	pub fn game(&self, game: &ModGame) -> ModActionData {
		let mut action = self.action(ModActionKind::ShowGame, ActionKind::None, &game.game_id);
		action.subject = game.title.as_str().into();
		action.provider = game.provider_id.into();
		action
	}

	pub fn push(&mut self, group: ActionGroup, mut action: ModActionData) {
		action.separator_before = self.group.is_some_and(|current| current != group);
		self.group = Some(group);
		self.actions.push(action);
	}

	pub fn finish(self) -> Vec<ModActionData> {
		self.actions
	}
}

pub fn select_actions(actions: &[ModActionData], kinds: &[ModActionKind]) -> Vec<ModActionData> {
	actions
		.iter()
		.filter(|action| kinds.contains(&action.kind))
		.cloned()
		.collect()
}
