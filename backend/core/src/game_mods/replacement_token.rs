use rai_pal_proc_macros::serializable_enum;

use crate::{
	app_paths,
	game::DbGame,
	game_mods::game_mod::GameMod,
	path_extensions::{
		AsValidStr,
		PathExt,
	},
	game_providers::provider_command::{
		ProviderCommand,
		ProviderCommandAction,
	},
	result::{
		Error,
		Result,
	},
};

#[serializable_enum]
pub enum ReplacementToken {
	GameExecutableFolderPath,
	GameExecutableName,
	GameExecutableNameWithoutExtension,
	GameExecutablePath,
	GameInstalledModsPath,
	GameJson,
	GameStartCommand,
	GameStartCommandArgs,
	SharedModsPath,
	MaybeWineRoot,
	RoamingAppData,
	// If adding new parameters, remember to update mod.ts in rai-pal-db repo.
	// (and bump the database version).
}

fn get_parameter_token(parameter: ReplacementToken) -> String {
	format!("{{{{{parameter}}}}}")
}

fn replace_parameter_value<TValue: AsRef<str>, TGetValue: Fn() -> Result<TValue>>(
	current: &str,
	token: ReplacementToken,
	get_value: TGetValue,
) -> String {
	if !current.contains(&get_parameter_token(token)) {
		return current.to_string();
	}

	match get_value() {
		Ok(value) => current.replace(&get_parameter_token(token), value.as_ref()),
		Err(error) => {
			log::error!(
				"Failed get value to replace token `{token}` in runnable argument `{current}`. Error: {error}"
			);
			current.to_string()
		}
	}
}

pub fn replace_tokens(base_string: &str, game: &DbGame, game_mod: &GameMod) -> String {
	let mut result = base_string.to_string();

	let provider_commands = &game.provider_commands.0;
	let start_command = provider_commands
		.get(&ProviderCommandAction::StartViaProvider)
		.or_else(|| provider_commands.get(&ProviderCommandAction::StartViaExe));

	result = replace_parameter_value(&result, ReplacementToken::GameExecutableName, || {
		game.try_get_exe_name()
	});
	result = replace_parameter_value(&result, ReplacementToken::GameExecutableFolderPath, || {
		game.try_get_exe_path()?.try_parent()?.try_to_str()
	});
	result = replace_parameter_value(
		&result,
		ReplacementToken::GameExecutableNameWithoutExtension,
		|| game.try_get_exe_path()?.file_name_without_extension(),
	);
	result = replace_parameter_value(&result, ReplacementToken::GameExecutablePath, || {
		game.try_get_exe_path()?.try_to_str()
	});
	result = replace_parameter_value(&result, ReplacementToken::GameJson, || {
		Ok(serde_json::to_string(&game)?)
	});
	result =
		replace_parameter_value(
			&result,
			ReplacementToken::GameStartCommand,
			|| match start_command
				.ok_or_else(|| Error::GameNotInstalled(game.display_title.clone()))?
			{
				ProviderCommand::String(s) => Ok(s.clone()),
				ProviderCommand::Path(exe_path, _) => Ok(exe_path.try_to_str()?.to_string()),
			},
		);
	result = replace_parameter_value(&result, ReplacementToken::GameStartCommandArgs, || {
		start_command.map_or_else(
			|| Ok(String::new()),
			|provider_command| match provider_command {
				ProviderCommand::Path(_, args) => Ok(args.join(" ")),
				ProviderCommand::String(_) => Ok(String::new()),
			},
		)
	});
	result = replace_parameter_value(&result, ReplacementToken::RoamingAppData, || {
		#[cfg(target_os = "linux")]
		{
			use crate::operating_system::OperatingSystem;

			if let Some(run) = game_mod.run_for_game.as_ref()
				&& run.os == Some(OperatingSystem::Windows)
			{
				// If runnable mod host OS is windows and we're on Linux, that means Wine,
				// which means config dir is inside the prefix.

				use std::{
					path::PathBuf,
					process::Command,
				};

				use crate::game_providers::game_provider;

				let provider = game_provider::get_provider(game.provider_id)?;
				let prefix_path = provider.get_wine_prefix_path(game)?;

				let output = Command::new(&provider.get_wine_binary_path(game)?)
					.env("WINEPREFIX", &prefix_path)
					.arg("cmd")
					.arg("/C")
					.arg("echo %APPDATA%")
					.output()?;

				let win_path = str::from_utf8(&output.stdout)?.trim();

				// 2. Convert Windows path to Linux path manually
				// The format is always C:\users\username\AppData\Roaming
				// We replace 'C:\' with the actual path to drive_c
				let drive_c_path = PathBuf::from(format!("{}/drive_c", prefix_path.try_to_str()?));

				// Remove the 'C:\' prefix and replace backslashes with slashes
				let relative_path = win_path.replace("C:\\", "").replace('\\', "/");

				return Ok(drive_c_path.join(relative_path).try_to_str()?.to_string());
			}
		}

		Ok(app_paths::base_dirs()?
			.config_dir()
			.try_to_str()?
			.to_string())
	});
	result = replace_parameter_value(&result, ReplacementToken::GameInstalledModsPath, || {
		Ok(game.get_installed_mods_folder()?.try_to_str()?.to_string())
	});
	result = replace_parameter_value(&result, ReplacementToken::SharedModsPath, || {
		Ok(app_paths::shared_mods_path()?.try_to_str()?.to_string())
	});
	result = replace_parameter_value(&result, ReplacementToken::MaybeWineRoot, || {
		#[cfg(target_os = "linux")]
		return Ok("Z:".to_string());
		#[cfg(target_os = "windows")]
		return Ok("".to_string());
	});

	result
}
