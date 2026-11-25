use std::{
	cell::RefCell,
	collections::{HashMap, HashSet},
	path::Path,
	rc::Rc,
	sync::Arc,
	thread,
};

use rai_pal_backend::{
	analytics::Event,
	game::Game,
	game_engines::game_engine::{EngineVersionNumbers, EngineVersionRange},
	game_providers::game_provider::GameProviderId,
	local_database::{
		app_database::Db,
		mod_database::{ModDatabase, ModInfo},
	},
	mods::{
		game_mods::{GameMod, GameModGroups},
		manifest::{Mod, ModDependency, ModPathState, ModPathType, install_steps},
	},
	result::{Error, LogErrExt, Result},
};
use slint::ComponentHandle;

use crate::{
	ActionKind, AppTab, GameModData, MainWindow, MessageContext, ModActionData, ModActionKind,
	ProviderId, action_state, analytics,
	downloads::Tracker,
	mod_actions::{ActionGroup, ModActionList, select_actions},
	model, notifications, runtime,
	ui_models::GameDetailsModels,
};

use super::{CommandHandler, on_command};

#[derive(Default)]
struct LoadedMods {
	installed: Vec<GameModData>,
	game_specific: Vec<GameModData>,
	available: Vec<GameModData>,
	hidden: Vec<GameModData>,
	incompatible: Vec<GameModData>,
}

type PathStateMap = HashMap<(String, String), ModPathState>;

thread_local! {
	static PATH_STATES: RefCell<HashMap<(GameProviderId, String), PathStateMap>> =
		RefCell::new(HashMap::new());
}

fn game_key(game: &Game) -> (GameProviderId, String) {
	(game.provider_id, game.game_id.clone())
}

fn cached_path_state(game: &Game, mod_id: &str, path: &str) -> Option<ModPathState> {
	PATH_STATES.with(|cache| {
		cache
			.borrow()
			.get(&game_key(game))
			.and_then(|states| states.get(&(mod_id.to_string(), path.to_string())).cloned())
	})
}

fn store_path_states(game: &Game, states: PathStateMap) {
	PATH_STATES.with(|cache| {
		cache.borrow_mut().insert(game_key(game), states);
	});
}

impl CommandHandler {
	pub fn refresh_mods(&self, game: &Game) {
		self.apply_mods(game);
		refresh_path_states(self.window(), Arc::clone(&self.database), game.clone());
	}

	fn apply_mods(&self, game: &Game) {
		let Some(window) = self.ui() else {
			return;
		};

		apply_mods(&window, &self.models.game_details, &self.database, game);
	}

	pub fn install_mod(&self, game: Game, mod_id: String) {
		analytics::record(Event::InstallMod {
			mod_id: mod_id.clone(),
			tags: self.mod_tags(&mod_id),
			game: Some(game.display_title.clone()),
		});
		self.install_mod_inner(game, mod_id);
	}

	pub fn update_mod(&self, game: Game, mod_id: String) {
		analytics::record(Event::UpdateMod {
			mod_id: mod_id.clone(),
			tags: self.mod_tags(&mod_id),
			game: Some(game.display_title.clone()),
		});
		self.install_mod_inner(game, mod_id);
	}

	fn install_mod_inner(&self, game: Game, mod_id: String) {
		let database = Arc::clone(&self.database);
		let window = self.window();
		let downloads = self.downloads.clone();

		self.spawn_action(
			action_state::key(ActionKind::GameModInstall, mod_id.clone()),
			MessageContext::InstallMod,
			move || {
				let result = install_mods(&database, &game, &[mod_id], &downloads);

				if result.is_ok() {
					refresh_after_action(&window, &game);
				}

				result
			},
		);
	}

	pub fn update_all_mods(&self, game: Game) {
		let database = Arc::clone(&self.database);
		let window = self.window();
		let downloads = self.downloads.clone();

		self.spawn_action(
			action_state::key(ActionKind::GameModUpdateAll, ""),
			MessageContext::UpdateAllMods,
			move || {
				let outdated_ids = database
					.get_game_mods(&game.provider_id, &game.game_id)
					.map_err(|err| err.to_string())?
					.into_iter()
					.filter(|info| info.compatible && info.is_outdated && info.is_installed)
					.map(|info| info.mod_id)
					.collect::<Vec<_>>();

				analytics::record_all(outdated_ids.iter().filter_map(|mod_id| {
					let game_mod = database
						.get_mod(mod_id)
						.ok_or_log("Failed to load mod for analytics")?;
					Some(Event::UpdateMod {
						mod_id: game_mod.id,
						tags: game_mod.tags,
						game: Some(game.display_title.clone()),
					})
				}));

				let result = install_mods(&database, &game, &outdated_ids, &downloads);

				if result.is_ok() {
					refresh_after_action(&window, &game);
				}

				result
			},
		);
	}

	pub fn uninstall_mod(&self, game: Game, mod_id: String) {
		analytics::record(Event::UninstallMod {
			mod_id: mod_id.clone(),
			tags: self.mod_tags(&mod_id),
			game: Some(game.display_title.clone()),
		});

		let database = Arc::clone(&self.database);
		let window = self.window();

		self.spawn_action(
			action_state::key(ActionKind::GameModUninstall, mod_id.clone()),
			MessageContext::UninstallMod,
			move || {
				let result = runtime::block_on(async {
					database
						.try_get_installed_mod(&game.provider_id, &game.game_id, &mod_id)?
						.uninstall()
						.await?;
					database.refresh_installed_mods()
				});

				if result.is_ok() {
					refresh_after_action(&window, &game);
				}

				result
			},
		);
	}

	pub fn uninstall_all_mods(&self, game: Game) {
		let database = Arc::clone(&self.database);
		let window = self.window();

		self.spawn_action(
			action_state::key(ActionKind::GameModUninstallAll, ""),
			MessageContext::UninstallAllMods,
			move || {
				let result = runtime::block_on(async {
					let installed_mods = database.get_installed_mods_for_game(&game)?;

					analytics::record_all(installed_mods.iter().map(|installed_mod| {
						Event::UninstallMod {
							mod_id: installed_mod.game_mod.id.clone(),
							tags: installed_mod.game_mod.tags.clone(),
							game: Some(game.display_title.clone()),
						}
					}));

					for installed_mod in installed_mods {
						installed_mod.uninstall().await?;
					}

					game.delete_mods_folder().await?;
					database.refresh_installed_mods()
				});

				if result.is_ok() {
					refresh_after_action(&window, &game);
				}

				result
			},
		);
	}

	pub fn run_mod(&self, game: Game, mod_id: String) {
		analytics::record(Event::RunMod {
			mod_id: mod_id.clone(),
			tags: self.mod_tags(&mod_id),
			game: Some(game.display_title.clone()),
		});

		let database = Arc::clone(&self.database);

		self.spawn_action(
			action_state::key(ActionKind::GameModRun, mod_id.clone()),
			MessageContext::RunMod,
			move || {
				load_mod(&database, &game, &mod_id).and_then(|game_mod| game_mod.run(Some(&game)))
			},
		);
	}

	pub fn open_mod_path(&self, game: Game, mod_id: String, path_id: String, open_folder: bool) {
		let database = Arc::clone(&self.database);

		self.spawn_action(
			action_state::key(ActionKind::GameModOpenPath, mod_id.clone()),
			MessageContext::OpenModPath,
			move || {
				load_mod(&database, &game, &mod_id)
					.and_then(|game_mod| game_mod.open_path(&path_id, Some(&game), open_folder))
			},
		);
	}
}

fn load_mod(database: &Db, game: &Game, mod_id: &str) -> Result<Mod> {
	match database.get_mod(mod_id) {
		Ok(game_mod) => Ok(game_mod),
		Err(Error::ModNotFound(_)) => Ok(database
			.try_get_installed_mod(&game.provider_id, &game.game_id, mod_id)?
			.game_mod),
		Err(error) => Err(error),
	}
}

fn refresh_path_states(window: slint::Weak<MainWindow>, database: Arc<Db>, game: Game) {
	thread::spawn(move || {
		let Ok(groups) = GameModGroups::load(database.as_ref(), &game) else {
			return;
		};
		let mut states = PathStateMap::new();

		for entry in groups
			.installed
			.iter()
			.chain(groups.game_specific.iter())
			.chain(groups.available.iter())
			.chain(groups.hidden.iter())
			.chain(groups.incompatible.iter())
		{
			let manifest = &entry.manifest;
			let Some(paths) = manifest.paths.as_ref() else {
				continue;
			};

			for path in paths {
				if let Ok(state) = manifest.get_path_state(&path.path, Some(&game)) {
					states.insert((manifest.id.clone(), path.path.clone()), state);
				}
			}
		}

		let game = game.clone();

		crate::runtime::with_ui_window(&window, move |_window| {
			store_path_states(&game, states);

			if let Some(handler) = CommandHandler::current() {
				handler.apply_mods(&game);
			}
		});
	});
}

fn apply_mods(window: &MainWindow, models: &GameDetailsModels, database: &Db, game: &Game) {
	let state = window.global::<crate::GameDetailsState>();
	state.set_mods_folder_exists(game.mods_folder_exists());

	let loaded = match load_mods(database, game) {
		Ok(loaded) => loaded,
		Err(err) => {
			log::error!("Failed to load mods: {err}");
			notifications::error(window, notifications::data(MessageContext::LoadMods, err));
			return;
		}
	};
	let has_outdated_mods = loaded
		.installed
		.iter()
		.chain(loaded.hidden.iter())
		.any(|mod_data| mod_data.outdated);
	let LoadedMods {
		installed,
		game_specific,
		available,
		hidden,
		incompatible,
	} = loaded;
	state.set_has_outdated_mods(has_outdated_mods);
	model::sync_vec_model(&models.installed, installed);
	model::sync_vec_model(&models.game_specific, game_specific);
	model::sync_vec_model(&models.available, available);
	model::sync_vec_model(&models.hidden, hidden);
	model::sync_vec_model(&models.incompatible, incompatible);
}

fn load_mods(database: &Db, game: &Game) -> Result<LoadedMods> {
	let list = GameModGroups::load(database, game)?;

	let installed = list
		.installed
		.into_iter()
		.map(|entry| row_data(entry, game))
		.collect::<Vec<_>>();
	let game_specific = list
		.game_specific
		.into_iter()
		.map(|entry| row_data(entry, game))
		.collect::<Vec<_>>();
	let available = list
		.available
		.into_iter()
		.map(|entry| row_data(entry, game))
		.collect::<Vec<_>>();
	let hidden = list
		.hidden
		.into_iter()
		.map(|entry| row_data(entry, game))
		.collect::<Vec<_>>();
	let incompatible = list
		.incompatible
		.into_iter()
		.map(|entry| row_data(entry, game))
		.collect::<Vec<_>>();
	Ok(LoadedMods {
		installed,
		game_specific,
		available,
		hidden,
		incompatible,
	})
}

fn row_data(entry: GameMod, game: &Game) -> GameModData {
	let GameMod { manifest, info } = entry;
	let installed = info.is_installed;
	let can_show_actions = info.compatible && game.exe_path.is_some();
	let has_install = manifest.install.is_some();
	let has_run = manifest.run_for_game.is_some();
	let installable =
		manifest.hide_from_game_mods_list != Some(true) || info.has_installed_dependants;
	let version_text = version_text(&info, &manifest);
	let release_date = release_date_display(&manifest);
	let engine_text = engine_text(&manifest);
	let mod_id = manifest.id.as_str();
	let mut actions = ModActionList::new(mod_id);

	for path in manifest.paths.as_deref().unwrap_or_default() {
		let state = cached_path_state(game, mod_id, &path.path);
		let display_path = match &state {
			Some(state) => state.path.as_path(),
			None => Path::new(&path.path),
		};
		let exists = |open_folder| {
			state
				.as_ref()
				.map_or(game.exe_path.is_some(), |state| state.exists(open_folder))
		};
		let display_name = |open_folder| {
			let target = if matches!(&path.path_type, ModPathType::File) && open_folder {
				display_path.parent().unwrap_or(display_path)
			} else {
				display_path
			};

			target.file_name().map_or_else(
				|| target.display().to_string(),
				|name| name.to_string_lossy().into_owned(),
			)
		};
		let variants = if matches!(&path.path_type, ModPathType::Folder) {
			vec![(true, exists(true))]
		} else {
			vec![(false, exists(false)), (true, exists(true))]
		};

		for (open_folder, exists) in variants {
			let mut action = actions.action(
				ModActionKind::OpenPath,
				ActionKind::GameModOpenPath,
				&path.path,
			);
			action.subject = display_name(open_folder).into();
			action.open_folder = open_folder;
			action.enabled = exists;
			actions.push(ActionGroup::Path, action);
		}
	}

	if can_show_actions && has_install && installable && !installed && !info.is_outdated {
		actions.push(
			ActionGroup::Primary,
			actions.action(ModActionKind::Install, ActionKind::GameModInstall, ""),
		);
	}

	if can_show_actions && info.is_outdated {
		actions.push(
			ActionGroup::Primary,
			actions.action(ModActionKind::Update, ActionKind::GameModInstall, ""),
		);
	}

	if can_show_actions && has_run && (!has_install || installed) {
		actions.push(
			ActionGroup::Primary,
			actions.action(ModActionKind::Run, ActionKind::GameModRun, ""),
		);
	}

	if can_show_actions && installed && has_install && info.is_in_catalog {
		actions.push(
			ActionGroup::Manage,
			actions.action(ModActionKind::Reinstall, ActionKind::GameModInstall, ""),
		);

		let mut uninstall =
			actions.action(ModActionKind::Uninstall, ActionKind::GameModUninstall, "");
		uninstall.enabled = !info.has_installed_dependants;
		actions.push(ActionGroup::Manage, uninstall);
	}

	if let Some(website) = &manifest.website {
		actions.push(
			ActionGroup::Link,
			actions.action(ModActionKind::Website, ActionKind::None, website),
		);
	}

	actions.push(
		ActionGroup::Debug,
		actions.action(ModActionKind::DebugData, ActionKind::None, ""),
	);

	let actions = actions.finish();
	let mut special_kinds = vec![
		ModActionKind::Install,
		ModActionKind::Update,
		ModActionKind::Run,
	];
	if !installed && !has_install && !has_run && manifest.website.is_some() {
		special_kinds.push(ModActionKind::Website);
	}
	let special_actions = select_actions(&actions, &special_kinds);

	GameModData {
		title: manifest.title.into(),
		description: manifest.description.into(),
		version_text: version_text.into(),
		release_date: release_date.into(),
		author: manifest.author.into(),
		engine_text: engine_text.into(),
		installed,
		outdated: info.is_outdated,
		deprecated: manifest.deprecated == Some(true),
		has_install,
		has_run,
		actions: Rc::new(slint::VecModel::from(actions)).into(),
		special_actions: Rc::new(slint::VecModel::from(special_actions)).into(),
	}
}

fn version_text(info: &ModInfo, game_mod: &Mod) -> String {
	let download_id = game_mod
		.download
		.as_ref()
		.map(|download| download.id.as_str());

	let mut parts = Vec::new();

	if let Some(version) = info.installed_version.as_deref().or(download_id) {
		parts.push(version);
	}

	if info.is_outdated
		&& let Some(download_id) = download_id
	{
		parts.push(download_id);
	}

	parts.join(" ➔ ")
}

pub fn release_date_display(game_mod: &Mod) -> String {
	crate::model::ReleaseDate::parse(game_mod.release_date()).display_short()
}

pub fn download_version(game_mod: &Mod) -> String {
	game_mod
		.download
		.as_ref()
		.map_or_default(|download| download.id.clone())
}

pub fn engine_text(game_mod: &Mod) -> String {
	let mut parts = Vec::new();

	if let Some(engine) = game_mod.engine {
		parts.push(engine.to_string());
	}

	if let Some(range) = &game_mod.engine_version_range
		&& let Some(range) = version_range(range)
	{
		parts.push(range);
	}

	if let Some(unity_backend) = game_mod.unity_backend {
		parts.push(unity_backend.to_string());
	}

	if let Some(game_os) = game_mod.game_os {
		parts.push(game_os.to_string());
	}

	if let Some(architecture) = game_mod.architecture {
		parts.push(architecture.to_string());
	}

	parts.join(" ")
}

fn version_range(range: &EngineVersionRange) -> Option<String> {
	let versions = [range.minimum.as_ref(), range.maximum.as_ref()]
		.into_iter()
		.flatten()
		.map(version_numbers)
		.collect::<Vec<_>>();

	if versions.is_empty() {
		None
	} else {
		Some(versions.join(" - "))
	}
}

fn version_numbers(version: &EngineVersionNumbers) -> String {
	let mut parts = vec![version.major.to_string()];

	if let Some(minor) = version.minor {
		parts.push(minor.to_string());
	}

	if let Some(patch) = version.patch {
		parts.push(patch.to_string());
	}

	parts.join(".")
}

fn collect_install_mods(
	database: &Db,
	mod_id: &str,
	relevant_mods: &[ModInfo],
) -> Result<(Mod, Vec<Mod>)> {
	let game_mod = database.get_mod(mod_id)?;
	let mut deps_to_install = Vec::new();

	collect_deps_to_install(
		mod_id,
		database,
		relevant_mods,
		&mut HashSet::new(),
		&mut deps_to_install,
	);

	Ok((game_mod, deps_to_install))
}

fn install_mods(
	database: &Db,
	game: &Game,
	mod_ids: &[String],
	downloads: &Tracker,
) -> std::result::Result<(), String> {
	if mod_ids.is_empty() {
		return Ok(());
	}

	let relevant_mods = database
		.get_game_mods(&game.provider_id, &game.game_id)
		.map_err(|err| err.to_string())?;

	for mod_id in mod_ids {
		let (game_mod, deps_to_install) = collect_install_mods(database, mod_id, &relevant_mods)
			.map_err(|err| err.to_string())?;

		let all_owned = deps_to_install
			.iter()
			.chain(std::iter::once(&game_mod))
			.cloned()
			.collect::<Vec<_>>();
		let steps = install_steps(&all_owned);
		downloads.pending_steps(&steps);

		let result = runtime::block_on(install_mod(
			database,
			game,
			game_mod,
			deps_to_install,
			downloads,
		));

		match &result {
			Ok(()) => downloads.finish_steps(&steps),
			Err(err) => {
				downloads.fail_steps(&steps);
				return Err(err.clone());
			}
		}
	}

	Ok(())
}

async fn install_mod(
	database: &Db,
	game: &Game,
	game_mod: Mod,
	deps_to_install: Vec<Mod>,
	downloads: &Tracker,
) -> Result {
	let mod_id = game_mod.id.clone();

	for dependency in &deps_to_install {
		dependency
			.install(Some(game), |status| downloads.record(&status))
			.await?;
	}

	if let Some(installed_mod) =
		database.get_installed_mod(&mod_id, Some(game.provider_id), Some(game.game_id.clone()))?
	{
		installed_mod.uninstall().await?;
	}

	if game_mod.install.is_some() {
		game_mod
			.install(Some(game), |status| downloads.record(&status))
			.await?;
	}

	database.refresh_installed_mods()?;

	Ok(())
}

fn collect_deps_to_install(
	mod_id: &str,
	database: &Db,
	relevant_mods: &[ModInfo],
	visited: &mut HashSet<String>,
	result: &mut Vec<Mod>,
) {
	if !visited.insert(mod_id.to_string()) {
		return;
	}

	let Some(game_mod) = database
		.get_mod(mod_id)
		.ok_or_log("Failed to load mod dependencies")
	else {
		return;
	};

	for dependencies in [
		game_mod.optional_dependencies.as_ref(),
		game_mod.required_dependencies.as_ref(),
	]
	.into_iter()
	.flatten()
	{
		for dependency in dependencies {
			let matching_mods = match dependency {
				ModDependency::ModId { mod_id: dep_id } => relevant_mods
					.iter()
					.filter(|info| info.compatible && info.mod_id == *dep_id)
					.collect::<Vec<_>>(),
				ModDependency::Tag { tag: dep_tag } => relevant_mods
					.iter()
					.filter(|info| info.compatible && info.tags.contains(dep_tag))
					.collect::<Vec<_>>(),
			};

			for info in matching_mods {
				let outdated = info.installed_version.is_none() || info.is_outdated;

				if outdated
					&& let Some(dep_mod) = database
						.get_mod(&info.mod_id)
						.ok_or_log("Failed to load dependency mod")
					&& dep_mod.install.is_some()
				{
					collect_deps_to_install(&info.mod_id, database, relevant_mods, visited, result);
					result.push(dep_mod);
				}
			}
		}
	}
}

fn refresh_after_action(window: &slint::Weak<MainWindow>, game: &Game) {
	let window = window.clone();
	let game = game.clone();

	crate::runtime::with_ui_window(&window, move |_window| {
		if let Some(handler) = CommandHandler::current() {
			handler.refresh_mods(&game);
		}
	});
}

impl CommandHandler {
	fn with_selected_mod(&self, mod_id: &str, action: impl FnOnce(&Self, Game, String)) {
		let Some(game) = self.selected_game() else {
			return;
		};

		action(self, game, mod_id.to_string());
	}

	pub fn activate_mod_action(&self, action: ModActionData) {
		let mod_id = action.mod_id.to_string();

		match action.kind {
			ModActionKind::Install => {
				self.prompt_install_mod(mod_id, crate::SkipConfirmation::InstallMod);
			}
			ModActionKind::Update => self.with_selected_mod(&mod_id, Self::update_mod),
			ModActionKind::Run => self.with_selected_mod(&mod_id, Self::run_mod),
			ModActionKind::Reinstall => {
				self.with_selected_mod(&mod_id, Self::install_mod);
			}
			ModActionKind::Uninstall => {
				self.with_selected_mod(&mod_id, Self::uninstall_mod);
			}
			ModActionKind::OpenPath => {
				self.with_selected_mod(&mod_id, |handler, game, mod_id| {
					handler.open_mod_path(
						game,
						mod_id,
						action.target.to_string(),
						action.open_folder,
					);
				});
			}
			ModActionKind::Website => super::actions::open_url(action.target.to_string()),
			ModActionKind::Download => self.install_standalone_mod(mod_id),
			ModActionKind::RunStandalone => self.run_standalone_mod(mod_id),
			ModActionKind::ShowGame => self.show_mod_game(action.provider, &action.target),
			ModActionKind::DebugData => self.show_mod_debug_data(&mod_id),
		}
	}

	fn show_mod_game(&self, provider: ProviderId, game_id: &str) {
		let Some(provider_id) = Option::<GameProviderId>::from(provider) else {
			return;
		};

		self.switch_tab(AppTab::Games);
		self.select_game_by_id(provider_id, game_id);
	}
}

pub fn register(window: &crate::MainWindow) {
	on_command!(window, ModMenuActions, on_activate, |action| handler => {
		handler.activate_mod_action(action);
	});
	on_command!(window, ModActions, on_update, |mod_id| handler => {
		handler.with_selected_mod(mod_id.as_str(), CommandHandler::update_mod);
	});
	on_command!(window, ModActions, on_reinstall, |mod_id| handler => {
		handler.with_selected_mod(mod_id.as_str(), CommandHandler::install_mod);
	});
	on_command!(window, ModActions, on_uninstall, |mod_id| handler => {
		handler.with_selected_mod(mod_id.as_str(), CommandHandler::uninstall_mod);
	});
	on_command!(window, ModActions, on_run, |mod_id| handler => {
		handler.with_selected_mod(mod_id.as_str(), CommandHandler::run_mod);
	});
	on_command!(window, ModActions, on_open_path, |mod_id, path_id, open_folder| handler => {
		handler.with_selected_mod(mod_id.as_str(), |handler, game, mod_id| {
			handler.open_mod_path(game, mod_id, path_id.to_string(), open_folder);
		});
	});
}
