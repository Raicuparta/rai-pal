use rai_pal_proc_macros::serializable_enum;

use crate::{
	game::DbGame,
	paths,
	providers::provider_command::{
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
	InstalledModsPath,
	LocalModsPath,
	GameExecutableFolderPath,
	GameExecutablePath,
	GameExecutableName,
	GameExecutableNameWithoutExtension,
	RoamingAppData,
	GameStartCommand,
	GameStartCommandArgs,
	GameJson,
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

pub fn replace_tokens(base_string: &str, game: &DbGame) -> String {
	let mut result = base_string.to_string();

	let provider_commands = &game.provider_commands.0;
	let start_command = provider_commands
		.get(&ProviderCommandAction::StartViaProvider)
		.or_else(|| provider_commands.get(&ProviderCommandAction::StartViaExe));

	result = replace_parameter_value(&result, ReplacementToken::GameExecutableName, || {
		game.try_get_exe_name()
	});
	result = replace_parameter_value(&result, ReplacementToken::GameExecutableFolderPath, || {
		Ok(game
			.try_get_exe_path()?
			.parent()
			.ok_or_else(|| Error::GameNotInstalled(game.display_title.clone()))?
			.to_string_lossy()
			.to_string())
	});
	result = replace_parameter_value(
		&result,
		ReplacementToken::GameExecutableNameWithoutExtension,
		|| paths::file_name_without_extension(game.try_get_exe_path()?),
	);
	result = replace_parameter_value(&result, ReplacementToken::GameExecutablePath, || {
		Ok(game.try_get_exe_path()?.to_string_lossy().to_string())
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
				ProviderCommand::Path(exe_path, _) => Ok(exe_path.to_string_lossy().to_string()),
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
		Ok(paths::base_dirs()?
			.config_dir()
			.to_string_lossy()
			.to_string())
	});
	result = replace_parameter_value(&result, ReplacementToken::InstalledModsPath, || {
		Ok(paths::installed_mods_path()?
			.join(&game.game_id)
			.to_string_lossy()
			.to_string())
	});
	result = replace_parameter_value(&result, ReplacementToken::LocalModsPath, || {
		Ok(paths::local_mods_path()?.to_string_lossy().to_string())
	});

	result
}
