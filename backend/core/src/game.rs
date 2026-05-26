use std::{
	collections::{
		HashMap,
		HashSet,
	},
	fs,
	path::{
		Path,
		PathBuf,
	},
};

use rai_pal_proc_macros::serializable_struct;

use crate::{
	app_paths,
	architecture::Architecture,
	data_types::{
		json_data::JsonData,
		path_data::PathData,
	},
	game_engines::{
		game_engine::{
			EngineBrand,
			EngineVersionRange,
			get_exe_engine,
		},
		unity::{
			self,
			UnityBackend,
		},
		unreal,
	},
	game_mods::{
		game_mod::GameMod,
		installed_mod::InstalledMod,
	},
	game_tag::GameTag,
	game_title::is_probably_demo,
	path_extensions::PathExt,
	providers::{
		provider::ProviderId,
		provider_command::{
			ProviderCommand,
			ProviderCommandAction,
		},
	},
	remote_config::{
		self,
		RemoteConfigs,
	},
	result::{
		Error,
		Result,
	},
};

#[serializable_struct]
pub struct GameModInfo {
	pub mod_id: String,
	pub installed_version: Option<String>,
	pub installed_hash: Option<String>,
	pub compatible: bool,
}

#[derive(serde::Serialize, specta::Type, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DbGame {
	pub provider_id: ProviderId,
	pub game_id: String,
	pub external_id: String,
	pub display_title: String,
	pub title_discriminator: Option<String>,
	pub thumbnail_url: Option<String>,
	pub release_date_rfc3339: Option<String>,
	pub exe_path: Option<PathData>,
	pub engine_brand: Option<EngineBrand>,
	pub engine_version_major: Option<u32>,
	pub engine_version_minor: Option<u32>,
	pub engine_version_patch: Option<u32>,
	pub engine_version_display: Option<String>,
	pub unity_backend: Option<UnityBackend>,
	pub architecture: Option<Architecture>,
	pub tags: JsonData<Vec<GameTag>>,
	pub provider_commands: JsonData<HashMap<ProviderCommandAction, ProviderCommand>>,
}

impl DbGame {
	pub fn new(provider_id: ProviderId, game_id: String, title: String) -> Self {
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
			tags: JsonData(Vec::default()),
			provider_commands: JsonData(HashMap::default()),
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

	pub fn get_manifest_paths(&self) -> Vec<PathBuf> {
		match self.get_installed_mod_manifest_path("*") {
			Ok(manifests_path) => {
				if !manifests_path.parent().is_some_and(Path::exists) {
					return Vec::default();
				}
				manifests_path.glob()
			}
			Err(err) => {
				log::error!(
					"Failed to get mod manifests glob path for game {}. Error: {}",
					self.display_title,
					err
				);
				Vec::default()
			}
		}
	}

	const fn is_engine_version_compatible(&self, range: Option<&EngineVersionRange>) -> bool {
		let Some(major) = self.engine_version_major else {
			return true;
		};
		let Some(range) = range else {
			return true;
		};

		let minor = self.engine_version_minor;
		let patch = self.engine_version_patch;

		if let Some(min) = &range.minimum {
			if min.major > major {
				return false;
			}
			if min.major == major
				&& let (Some(min_minor), Some(game_minor)) = (min.minor, minor)
			{
				if min_minor > game_minor {
					return false;
				}
				if min_minor == game_minor
					&& let (Some(min_patch), Some(game_patch)) = (min.patch, patch)
					&& min_patch > game_patch
				{
					return false;
				}
			}
		}

		if let Some(max) = &range.maximum {
			if max.major < major {
				return false;
			}
			if max.major == major
				&& let (Some(max_minor), Some(game_minor)) = (max.minor, minor)
			{
				if max_minor < game_minor {
					return false;
				}
				if max_minor == game_minor
					&& let (Some(max_patch), Some(game_patch)) = (max.patch, patch)
					&& max_patch < game_patch
				{
					return false;
				}
			}
		}

		true
	}

	pub fn get_relevant_mods(
		&self,
		local_mods: &HashMap<String, GameMod>,
		remote_mods: &HashMap<String, GameMod>,
	) -> Vec<GameModInfo> {
		// Refresh installed manifests and preserve update-on-read behavior
		let installed_manifests: HashMap<String, GameMod> = self
			.get_manifest_paths()
			.iter()
			.filter_map(|manifest_path| {
				let mut manifest = GameMod::from_file(manifest_path)?;

				if let Some(local_mod) = local_mods.get(&manifest.id)
					&& manifest.latest_version.id == local_mod.latest_version.id
					&& manifest.hash != local_mod.hash
					&& let Ok(manifest_contents) = serde_json::to_string_pretty(local_mod)
				{
					let _ = fs::write(manifest_path, &manifest_contents);

					if let Ok(updated_manifest) =
						serde_json::from_str::<GameMod>(&manifest_contents)
					{
						manifest = updated_manifest;
					}
				}

				Some((manifest_path.file_stem()?.to_str()?.to_string(), manifest))
			})
			.collect();

		// Collect all mod IDs from local and remote (prefer local data over remote)
		let all_mod_ids: HashSet<&String> = local_mods.keys().chain(remote_mods.keys()).collect();

		let mut result = Vec::new();

		for mod_id in all_mod_ids {
			let mod_data = local_mods
				.get(mod_id)
				.or_else(|| remote_mods.get(mod_id))
				.unwrap();

			// Skip if mod requires a specific engine and game's engine doesn't match
			if let Some(required_engine) = &mod_data.engine
				&& self.engine_brand.as_ref() != Some(required_engine)
			{
				continue;
			}

			// Skip if both mod and game specify unity_backend and they differ
			if let (Some(mod_backend), Some(game_backend)) =
				(&mod_data.unity_backend, &self.unity_backend)
				&& mod_backend != game_backend
			{
				continue;
			}

			// Skip if both mod and game specify architecture and they differ
			if let (Some(mod_arch), Some(game_arch)) = (&mod_data.architecture, &self.architecture)
				&& mod_arch != game_arch
			{
				continue;
			}

			let installed_manifest = installed_manifests.get(mod_id.as_str());

			// Skip deprecated mods unless they are installed
			if mod_data.deprecated == Some(true) && installed_manifest.is_none() {
				continue;
			}

			// Skip mods that are neither installable nor runnable
			if mod_data.install.is_none() && mod_data.run_for_game.is_none() {
				continue;
			}

			let compatible =
				self.is_engine_version_compatible(mod_data.engine_version_range.as_ref());

			let (installed_version, installed_hash) = installed_manifest
				.map_or((None, None), |m| {
					(Some(m.latest_version.id.clone()), m.hash.clone())
				});

			result.push(GameModInfo {
				mod_id: mod_id.clone(),
				installed_version,
				installed_hash,
				compatible,
			});
		}

		result
	}

	pub fn get_installed_mod_manifest_path(&self, mod_id: &str) -> Result<PathBuf> {
		Ok(self
			.get_installed_mods_folder()?
			.join("manifests")
			.join(format!("{mod_id}.json")))
	}

	pub fn get_installed_mods_folder(&self) -> Result<PathBuf> {
		let installed_mods_folder =
			app_paths::installed_mods_path()?.join(self.try_get_exe_path()?.hash_string());
		fs::create_dir_all(&installed_mods_folder)?;

		Ok(installed_mods_folder)
	}

	pub fn try_get_exe_path(&self) -> Result<&Path> {
		Ok(&self
			.exe_path
			.as_ref()
			.ok_or_else(|| Error::GameNotInstalled(self.display_title.clone()))?
			.0)
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
		self.provider_commands.0.insert(command_action, command);
		self
	}

	pub fn add_tag(&mut self, tag: GameTag) -> &mut Self {
		if self.tags.0.contains(&tag) {
			return self;
		}

		self.tags.0.push(tag);
		self
	}

	pub fn set_executable(&mut self, exe_path: &Path) -> &mut Self {
		const VALID_EXTENSIONS: [&str; 3] = ["exe", "x86_64", "x86"];

		if !exe_path.is_file() {
			return self;
		}

		// TODO: Launching exes directly only works on Windows.
		// Once we have native Linux executable support, we should change this based on the executable's format and current OS.
		#[cfg(target_os = "windows")]
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

			self.exe_path = Some(PathData(exe_path.normalize()));
			if let Some(exe_engine_brand) = get_exe_engine(exe_path) {
				self.engine_brand = Some(exe_engine_brand);
				match exe_engine_brand {
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
		if let Some(PathData(exe_path)) = self.exe_path.clone() {
			self.set_executable(&exe_path);
		} else {
			return Err(Error::GameNotInstalled(self.display_title.clone()));
		}
		Ok(self)
	}

	pub async fn get_remote_configs(&self) -> Result<Option<RemoteConfigs>> {
		if let Some(exe_path) = self.exe_path.as_ref() {
			remote_config::get_remote_configs(&exe_path.0).await
		} else {
			Err(Error::GameNotInstalled(self.display_title.clone()))
		}
	}

	pub fn get_installed_mod(&self, mod_id: &str) -> Result<Option<InstalledMod<'_>>> {
		Ok(
			GameMod::from_file(&self.get_installed_mod_manifest_path(mod_id)?).map(|game_mod| {
				InstalledMod {
					game_mod,
					game: self,
				}
			}),
		)
	}

	pub fn try_get_installed_mod(&self, mod_id: &str) -> Result<InstalledMod<'_>> {
		self.get_installed_mod(mod_id)?
			.ok_or(Error::ModNotInstalled(mod_id.to_string()))
	}
}
