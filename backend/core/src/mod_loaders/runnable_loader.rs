use std::{
	path::{Path, PathBuf},
	process::Command,
};

use log::error;
use rai_pal_proc_macros::{serializable_enum, serializable_struct};

use super::mod_loader::{ModLoaderActions, ModLoaderData, ModLoaderStatic};
use crate::{
	game::DbGame,
	game_mod::CommonModData,
	local_mod::{self, LocalMod, ModKind},
	mod_loaders::mod_database::ModConfigs,
	mod_manifest,
	paths::{self, glob_path},
	providers::provider_command::{ProviderCommand, ProviderCommandAction},
	result::{Error, Result},
};

#[serializable_struct]
pub struct RunnableLoader {
	pub data: ModLoaderData,
}

#[serializable_enum]
pub enum RunnableParameter {
	ExecutableName,
	ExecutableNameWithoutExtension,
	ExecutablePath,
	GameJson,
	StartCommand,
	StartCommandArgs,
	RoamingAppData,
	// If adding new parameters, remember to update runnable_schema.json in rai-pal-db repo.
}

impl ModLoaderStatic for RunnableLoader {
	const ID: &'static str = "runnable";

	fn new(resources_path: &Path) -> Result<Self>
	where
		Self: std::marker::Sized,
	{
		Ok(Self {
			data: ModLoaderData {
				id: Self::ID.to_string(),
				path: resources_path.join(Self::ID),
				kind: ModKind::Runnable,
			},
		})
	}
}

fn get_parameter_token(parameter: RunnableParameter) -> String {
	format!("{{{{{parameter}}}}}")
}

fn replace_parameter_value<TValue: AsRef<str>, TGetValue: Fn() -> Result<TValue>>(
	argument: &str,
	parameter: RunnableParameter,
	get_value: TGetValue,
) -> String {
	if !argument.contains(&get_parameter_token(parameter)) {
		return argument.to_string();
	}

	match get_value() {
		Ok(value) => argument.replace(&get_parameter_token(parameter), value.as_ref()),
		Err(error) => {
			error!(
				"Failed get value to replace parameter `{parameter}` in runnable argument `{argument}`. Error: {error}"
			);
			argument.to_string()
		}
	}
}

fn replace_parameters(base_string: &str, game: &DbGame) -> String {
	let mut result = base_string.to_string();

	let provider_commands = &game.provider_commands.0;
	let start_command = provider_commands
		.get(&ProviderCommandAction::StartViaProvider)
		.or_else(|| provider_commands.get(&ProviderCommandAction::StartViaExe));

	result = replace_parameter_value(&result, RunnableParameter::ExecutableName, || {
		game.try_get_exe_name()
	});
	result = replace_parameter_value(
		&result,
		RunnableParameter::ExecutableNameWithoutExtension,
		|| paths::file_name_without_extension(game.try_get_exe_path()?),
	);
	result = replace_parameter_value(&result, RunnableParameter::ExecutablePath, || {
		Ok(game.try_get_exe_path()?.to_string_lossy().to_string())
	});
	result = replace_parameter_value(&result, RunnableParameter::GameJson, || {
		Ok(serde_json::to_string(&game)?)
	});
	result =
		replace_parameter_value(
			&result,
			RunnableParameter::StartCommand,
			|| match start_command
				.ok_or_else(|| Error::GameNotInstalled(game.display_title.clone()))?
			{
				ProviderCommand::String(s) => Ok(s.to_string()),
				ProviderCommand::Path(exe_path, _) => Ok(exe_path.to_string_lossy().to_string()),
			},
		);
	result = replace_parameter_value(&result, RunnableParameter::StartCommandArgs, || {
		start_command.map_or_else(
			|| Ok(String::new()),
			|provider_command| match provider_command {
				ProviderCommand::Path(_, args) => Ok(args.join(" ")),
				ProviderCommand::String(_) => Ok(String::new()),
			},
		)
	});
	result = replace_parameter_value(&result, RunnableParameter::RoamingAppData, || {
		Ok(paths::base_dirs()?
			.config_dir()
			.to_string_lossy()
			.to_string())
	});

	result
}

impl ModLoaderActions for RunnableLoader {
	fn get_data(&self) -> &ModLoaderData {
		&self.data
	}

	fn install(&self, _game: &DbGame) -> Result {
		todo!()
	}

	async fn install_mod_inner(&self, game: &DbGame, local_mod: &LocalMod) -> Result {
		let mod_folder = self.get_mod_path(&local_mod.common)?;

		let runnable = local_mod
			.data
			.manifest
			.as_ref()
			.and_then(|manifest| manifest.runnable.as_ref())
			.ok_or_else(|| Error::RunnableManifestNotFound(local_mod.common.id.clone()))?;

		let args: Vec<String> = runnable
			.args
			.iter()
			.map(|arg| replace_parameters(arg, game))
			.collect();

		Command::new(mod_folder.join(&runnable.path))
			.current_dir(mod_folder)
			.args(&args)
			.spawn()?;

		Ok(())
	}

	async fn uninstall_mod(&self, _game: &DbGame, _local_mod: &LocalMod) -> Result {
		// There's nothing to uninstall for runnables.

		Ok(())
	}

	async fn run_without_game(&self, local_mod: &LocalMod) -> Result {
		let mod_folder = self.get_mod_path(&local_mod.common)?;

		let runnable = local_mod
			.data
			.manifest
			.as_ref()
			.and_then(|manifest| manifest.runnable.as_ref())
			.ok_or_else(|| Error::RunnableManifestNotFound(local_mod.common.id.clone()))?;

		Command::new(mod_folder.join(&runnable.path))
			.current_dir(mod_folder)
			.spawn()?;

		Ok(())
	}

	fn open_installed_mod_folder(&self, _game: &DbGame, local_mod: &LocalMod) -> Result {
		let mod_folder = self.get_mod_path(&local_mod.common)?;

		Ok(open::that_detached(mod_folder)?)
	}

	fn get_mod_path(&self, mod_data: &CommonModData) -> Result<PathBuf> {
		Ok(Self::get_installed_mods_path()?.join(&mod_data.id))
	}

	fn get_local_mods(&self) -> Result<local_mod::Map> {
		let mods_path = Self::get_installed_mods_path()?;

		let mut mod_map = local_mod::Map::default();

		for manifest_path in glob_path(&mods_path.join("*").join(mod_manifest::Manifest::FILE_NAME))
		{
			if let Some(manifest) = mod_manifest::get(&manifest_path) {
				match LocalMod::new(
					Self::ID,
					manifest_path.parent().unwrap_or(&manifest_path),
					manifest.engine,
					manifest.unity_backend,
				) {
					Ok(local_mod) => {
						mod_map.insert(local_mod.common.id.clone(), local_mod);
					}
					Err(error) => {
						error!(
							"Failed to create local runnable mod from manifest in {}. Error: {}",
							manifest_path.display(),
							error
						);
					}
				}
			}
		}

		Ok(mod_map)
	}

	fn get_config_path(&self, game: &DbGame, mod_configs: &ModConfigs) -> Result<PathBuf> {
		Ok(PathBuf::from(replace_parameters(
			&mod_configs.destination_path,
			game,
		)))
	}
}
