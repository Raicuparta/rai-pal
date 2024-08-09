use std::{
	path::{Path, PathBuf},
	process::Command,
};

use log::error;
use rai_pal_proc_macros::serializable_struct;

use super::mod_loader::{ModLoaderActions, ModLoaderData, ModLoaderStatic};
use crate::{
	game_mod::CommonModData,
	installed_game::InstalledGame,
	local_mod::{self, LocalMod, ModKind},
	mod_manifest,
	paths::glob_path,
	providers::provider_command::ProviderCommand,
	result::Error,
	result::Result,
	serializable_enum,
};

#[serializable_struct]
pub struct RunnableLoader {
	pub data: ModLoaderData,
}

serializable_enum!(RunnableParameter {
	ExecutableName,
	ExecutablePath,
	GameJson,
	StartCommand,
	StartCommandArgs,
});

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
			error!("Failed get value to replace parameter `{parameter}` in runnable argument `{argument}`. Error: {error}");
			argument.to_string()
		}
	}
}

fn replace_parameters(argument: &str, game: &InstalledGame) -> String {
	let mut result = argument.to_string();

	result = replace_parameter_value(&result, RunnableParameter::ExecutableName, || {
		Ok(&game.executable.name)
	});
	result = replace_parameter_value(&result, RunnableParameter::ExecutablePath, || {
		Ok(game.executable.path.to_string_lossy())
	});
	result = replace_parameter_value(&result, RunnableParameter::GameJson, || {
		Ok(serde_json::to_string(&game)?)
	});
	result = replace_parameter_value(&result, RunnableParameter::StartCommand, || {
		game.start_command.as_ref().map_or_else(
			|| Ok(game.executable.path.to_string_lossy().to_string()),
			|provider_command| match provider_command {
				ProviderCommand::String(s) => Ok(s.to_string()),
				ProviderCommand::Path(exe_path, _) => Ok(exe_path.to_string_lossy().to_string()),
			},
		)
	});
	result = replace_parameter_value(&result, RunnableParameter::StartCommandArgs, || {
		game.start_command.as_ref().map_or_else(
			|| Ok(String::new()),
			|provider_command| match provider_command {
				ProviderCommand::Path(_, args) => Ok(args.join(" ")),
				ProviderCommand::String(_) => Ok(String::new()),
			},
		)
	});

	result
}

impl ModLoaderActions for RunnableLoader {
	fn get_data(&self) -> &ModLoaderData {
		&self.data
	}

	fn install(&self, _game: &InstalledGame) -> Result {
		todo!()
	}

	async fn install_mod_inner(&self, game: &InstalledGame, local_mod: &LocalMod) -> Result {
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

	async fn uninstall_mod(&self, _game: &InstalledGame, _local_mod: &LocalMod) -> Result {
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

	fn configure_mod(&self, game: &InstalledGame, local_mod: &LocalMod) -> Result {
		// TODO: make it actually open the config file / folder (would need extra info in database / manifest).
		self.open_installed_mod_folder(game, local_mod)
	}

	fn open_installed_mod_folder(&self, _game: &InstalledGame, local_mod: &LocalMod) -> Result {
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
}
