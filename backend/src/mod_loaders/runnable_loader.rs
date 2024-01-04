use std::{
	path::{
		Path,
		PathBuf,
	},
	process::Command,
};

use async_trait::async_trait;
use log::error;

use super::mod_loader::{
	ModLoaderActions,
	ModLoaderData,
	ModLoaderStatic,
};
use crate::{
	game_mod::CommonModData,
	installed_game::InstalledGame,
	local_mod::{
		self,
		LocalMod,
		ModKind,
	},
	mod_manifest,
	paths::glob_path,
	result::Error,
	serializable_enum,
	serializable_struct,
	Result,
};

serializable_struct!(RunnableLoader {
  pub data: ModLoaderData,
});

serializable_enum!(RunnableParameter {
	ExecutableName,
	ExecutablePath,
	GameJson,
});

#[async_trait]
impl ModLoaderStatic for RunnableLoader {
	const ID: &'static str = "runnable";

	async fn new(resources_path: &Path) -> Result<Self>
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

	result
}

#[async_trait]
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

	fn get_mod_path(&self, mod_data: &CommonModData) -> Result<PathBuf> {
		Ok(Self::get_installed_mods_path()?.join(&mod_data.id))
	}

	fn get_local_mods(&self) -> Result<local_mod::Map> {
		let mods_path = Self::get_installed_mods_path()?;

		let manifests = glob_path(
			&mods_path
				.join("*")
				// TODO manifest name const somewhere.
				.join("rai-pal-manifest.json"),
		)?;

		let mut mod_map = local_mod::Map::default();

		for manifest_path_result in manifests {
			match manifest_path_result {
				Ok(manifest_path) => {
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
				Err(error) => {
					error!(
						"Failed to read mod manifest from {}. Error: {}",
						mods_path.display(),
						error
					);
				}
			}
		}

		Ok(mod_map)
	}
}
