use std::collections::HashMap;

use rai_pal_core::game_providers::game_provider::GameProviderId;
use tauri::{
	AppHandle,
	Url,
};

use crate::{
	events::{
		self,
		EventEmitter,
	},
	install_mod,
	run_mod,
	uninstall_mod,
};

#[derive(Clone, Copy, Debug)]
enum ModAction {
	Install,
	Uninstall,
	Run,
}

pub fn handle(raw_url: &str, handle: &AppHandle) {
	let Ok(url) = Url::parse(raw_url) else {
		return;
	};

	if url.scheme() != "rai-pal" {
		return;
	}

	let Some(action) = url.host_str().map(str::to_string) else {
		return;
	};

	let query: HashMap<String, String> = url
		.query_pairs()
		.map(|(key, value)| (key.into_owned(), value.into_owned()))
		.collect();

	match action.as_str() {
		"add-mod-source" => {
			if let Some(source_url) = query.get("url") {
				handle.emit_safe(events::AddModSource(source_url.clone()));
			}
		}

		"install-mod" => dispatch_mod_action(ModAction::Install, &query, handle),

		"uninstall-mod" => dispatch_mod_action(ModAction::Uninstall, &query, handle),

		"run-mod" => dispatch_mod_action(ModAction::Run, &query, handle),

		_ => log::debug!("Ignoring unknown rai-pal:// path `{action}`"),
	}
}

fn dispatch_mod_action(action: ModAction, query: &HashMap<String, String>, handle: &AppHandle) {
	let Some(provider_id) = query
		.get("providerId")
		.and_then(|value| value.parse::<GameProviderId>().ok())
	else {
		log::warn!("Ignoring rai-pal:// {action:?} deep link without a valid providerId");
		return;
	};

	let Some(game_id) = query.get("gameId") else {
		log::warn!("Ignoring rai-pal:// {action:?} deep link without a gameId");
		return;
	};

	let Some(mod_id) = query.get("modId") else {
		log::warn!("Ignoring rai-pal:// {action:?} deep link without a modId");
		return;
	};

	let game_id = game_id.clone();
	let mod_id = mod_id.clone();
	let handle = handle.clone();

	tauri::async_runtime::spawn(async move {
		let result = match action {
			ModAction::Install => {
				install_mod(&mod_id, Some(provider_id), Some(game_id), handle).await
			}
			ModAction::Uninstall => uninstall_mod(provider_id, game_id, &mod_id, handle).await,
			ModAction::Run => run_mod(&mod_id, Some(provider_id), Some(game_id), handle).await,
		};

		if let Err(error) = result {
			log::error!("rai-pal:// {action:?} deep link failed: {error}");
		}
	});
}

