use std::{
	collections::BTreeMap,
	fs,
	path::{
		Path,
		PathBuf,
	},
};

use crate::{
	app_paths,
	architecture::Architecture,
	game_engines::{
		game_engine::{
			EngineBrand,
			get_exe_engine,
		},
		godot,
		unity::{
			self,
			UnityBackend,
		},
		unreal,
	},
	game_providers::{
		game_provider::GameProviderId,
		provider_command::{
			ProviderCommand,
			ProviderCommandAction,
		},
	},
	game_tag::GameTag,
	game_title::is_probably_demo,
	path_extensions::PathExt,
	remote_config::{
		self,
		RemoteConfigs,
	},
	result::{
		Error,
		Result,
	},
};

#[derive(serde::Serialize, specta::Type, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DbGame {
	pub provider_id: GameProviderId,
	pub game_id: String,
	pub external_id: String,
	pub display_title: String,
	pub title_discriminator: Option<String>,
	pub thumbnail_url: Option<String>,
	pub release_date_rfc3339: Option<String>,
	pub exe_path: Option<PathBuf>,
	pub engine_brand: Option<EngineBrand>,
	pub engine_version_major: Option<u32>,
	pub engine_version_minor: Option<u32>,
	pub engine_version_patch: Option<u32>,
	pub engine_version_display: Option<String>,
	pub unity_backend: Option<UnityBackend>,
	pub architecture: Option<Architecture>,
	pub tags: Vec<GameTag>,
	pub provider_commands: BTreeMap<ProviderCommandAction, ProviderCommand>,
}

impl DbGame {
	pub fn new(provider_id: GameProviderId, game_id: String, title: String) -> Self {
		let mut game = Self {
			provider_id,
			external_id: game_id.clone(),
			game_id,
			display_title: title,
			title_discriminator: None,
			thumbnail_url: None,
			release_date_rfc3339: None,
			exe_path: None,
			engine_brand: None,
			engine_version_major: None,
			engine_version_minor: None,
			engine_version_patch: None,
			engine_version_display: None,
			unity_backend: None,
			architecture: None,
			tags: Vec::default(),
			provider_commands: BTreeMap::default(),
		};

		if is_probably_demo(&game.display_title) {
			game.add_tag(GameTag::Demo);
		}

		game
	}

	pub fn open_game_folder(&self) -> Result {
		self.try_get_exe_path()?.open_folder_or_parent()
	}

	pub fn open_mods_folder(&self) -> Result {
		self.get_installed_mods_folder()?.open_folder_or_parent()
	}

	pub fn uninstall_all_mods(&self) -> Result {
		Ok(fs::remove_dir_all(self.get_installed_mods_folder()?)?)
	}

	pub fn get_installed_mods_folder(&self) -> Result<PathBuf> {
		let installed_mods_folder =
			app_paths::installed_mods_path()?.join(self.try_get_exe_path()?.hash_string());
		fs::create_dir_all(&installed_mods_folder)?;

		Ok(installed_mods_folder)
	}

	pub fn try_get_exe_path(&self) -> Result<&Path> {
		Ok(self
			.exe_path
			.as_ref()
			.ok_or_else(|| Error::GameNotInstalled(self.display_title.clone()))?)
	}

	pub fn try_get_exe_name(&self) -> Result<String> {
		let path = self.try_get_exe_path()?;
		path.file_name()
			.and_then(|file_name| file_name.to_str())
			.map(std::string::ToString::to_string)
			.ok_or_else(|| Error::InvalidOsStr(path.display().to_string()))
	}

	pub fn add_provider_command(
		&mut self,
		command_action: ProviderCommandAction,
		command: ProviderCommand,
	) -> &mut Self {
		self.provider_commands.insert(command_action, command);
		self
	}

	pub fn add_tag(&mut self, tag: GameTag) -> &mut Self {
		if self.tags.contains(&tag) {
			return self;
		}

		self.tags.push(tag);
		self
	}

	pub fn set_executable(&mut self, exe_path: &Path) -> &mut Self {
		const VALID_EXTENSIONS: [&str; 3] = ["exe", "x86_64", "x86"];

		if !exe_path.is_file() {
			return self;
		}

		self.add_provider_command(
			ProviderCommandAction::StartViaExe,
			ProviderCommand::Path(exe_path.to_path_buf(), Vec::default()),
		);

		// We ignore games that don't have an extension.
		if let Some(extension) = exe_path.extension().and_then(|ext| ext.to_str()) {
			if !VALID_EXTENSIONS.contains(&extension.to_lowercase().as_str()) {
				return self;
			}

			if extension == "x86" && exe_path.with_extension("x86_64").is_file() {
				// If there's an x86_64 version, we ignore the x86 version.
				// I'm just gonna presume there are no x86 modders out there,
				// if someone cries about it I'll make this smarter.
				return self;
			}

			self.exe_path = Some(exe_path.normalize());
			if let Some(exe_engine_brand) = get_exe_engine(exe_path) {
				self.engine_brand = Some(exe_engine_brand);
				match exe_engine_brand {
					EngineBrand::Godot => {
						godot::process_game(self);
					}
					EngineBrand::Unity => {
						unity::process_game(self);
					}
					EngineBrand::Unreal => {
						unreal::process_game(self);
					}
					_ => {}
				}
			}
		}

		self
	}

	pub fn refresh_executable(&mut self) -> Result<&mut Self> {
		if let Some(exe_path) = self.exe_path.clone() {
			self.set_executable(&exe_path);
		} else {
			return Err(Error::GameNotInstalled(self.display_title.clone()));
		}
		Ok(self)
	}

	pub async fn get_remote_configs(&self) -> Result<Option<RemoteConfigs>> {
		if let Some(exe_path) = self.exe_path.as_ref() {
			remote_config::get_remote_configs(exe_path).await
		} else {
			Err(Error::GameNotInstalled(self.display_title.clone()))
		}
	}
}
