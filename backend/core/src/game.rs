use rai_pal_proc_macros::serializable_struct;

use crate::{
	installed_game::InstalledGame, owned_game::OwnedGame, providers::provider::ProviderId,
};

#[serializable_struct]
pub struct Game {
	pub id: String,
	pub provider_id: ProviderId,
	pub installed_game: Option<InstalledGame>,
	pub owned_game: Option<OwnedGame>,
}
