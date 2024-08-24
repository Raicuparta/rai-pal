use std::path::Path;

use rai_pal_proc_macros::serializable_struct;
use serde::{Deserialize, Serialize};

use crate::{
	game_subscription::GameSubscription,
	installed_game::InstalledGame,
	owned_game::OwnedGame,
	providers::provider::{ProviderActions, ProviderId, ProviderStatic},
	result::Result,
};

use super::provider_command::{ProviderCommand, ProviderCommandAction};

#[derive(Clone)]
pub struct Ubisoft {}

impl ProviderStatic for Ubisoft {
	const ID: &'static ProviderId = &ProviderId::Ubisoft;

	fn new() -> Result<Self>
	where
		Self: Sized,
	{
		Ok(Self {})
	}
}

#[derive(Serialize, Deserialize, specta::Type, Clone, Debug)]
struct UbisoftPlusGame {
	title: String,
	id: String,
	#[serde(rename = "MasterID")]
	master_id: String,
	image_link: String,
	short_title: String,
	release_date: String,
	#[serde(rename = "partofSubscriptionOffer")]
	part_of_subscription_offer: Vec<String>,
}

#[serializable_struct]
struct UbisoftPlusGameDatabase {
	hits: Vec<UbisoftPlusGame>,
}

impl UbisoftPlusGame {
	fn get_release_date(&self) -> Option<i64> {
		Some(
			self.release_date
				.parse::<chrono::DateTime<chrono::Utc>>()
				.ok()?
				.timestamp(),
		)
	}
}

#[derive(serde::Serialize, serde::Deserialize, specta::Type, Clone, Debug)]
#[serde(rename_all = "PascalCase")]
struct UbisoftGamepassImages {
	logo: Option<Vec<String>>,
	box_art: Option<Vec<String>>,
}

impl ProviderActions for Ubisoft {
	async fn get_games<TInstalledCallback, TOwnedCallback>(
		&self,
		mut _installed_callback: TInstalledCallback,
		mut owned_callback: TOwnedCallback,
	) -> Result
	where
		TInstalledCallback: FnMut(InstalledGame) + Send + Sync,
		TOwnedCallback: FnMut(OwnedGame) + Send + Sync,
	{
		let json_path =
			Path::new(env!("CARGO_MANIFEST_DIR")).join("../../test-data/Ubisoft-plus-games.json");

		let database: UbisoftPlusGameDatabase =
			serde_json::from_str(&std::fs::read_to_string(json_path)?)?;

		for game in database.hits {
			let mut owned_game = OwnedGame::new(&game.id, *Self::ID, &game.title);

			owned_game.add_provider_command(
				ProviderCommandAction::OpenInBrowser,
				ProviderCommand::String(
					format!("https://store.ubisoft.com/us/a/{}.html", game.id,),
				),
			);

			if let Some(release_date) = game.get_release_date() {
				owned_game.set_release_date(release_date);
			}

			owned_game.set_thumbnail_url(&game.image_link);

			for subscription in game.part_of_subscription_offer {
				if subscription.to_lowercase().contains("premium") {
					owned_game.add_subscription(GameSubscription::UbisoftPremium);
				}
				if subscription.to_lowercase().contains("classics") {
					owned_game.add_subscription(GameSubscription::UbisoftClassics);
				}
			}

			owned_callback(owned_game);
		}

		Ok(())
	}
}
