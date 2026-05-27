use std::{
	collections::BTreeMap,
	fs,
	path::{
		Path,
		PathBuf,
	},
	process::Command,
};

use steamlocate::SteamDir;

use crate::{
	game::DbGame,
	path_extensions::PathExt,
	providers::{
		provider::WineProviderActions,
		steam::steam_provider::Steam,
	},
	result::{
		Error,
		Result,
	},
};

const DLL_OVERRIDES_SECTION: &str = "[Software\\\\Wine\\\\DllOverrides]";
const DLL_OVERRIDE_VALUE: &str = "native,builtin";

impl WineProviderActions for Steam {
	fn get_wine_prefix_path(&self, game: &DbGame) -> Result<PathBuf> {
		let app_id = game.external_id.parse()?;
		let steam_dir = SteamDir::locate()?;
		let (_, library) = steam_dir.find_app(app_id)?.ok_or_else(|| {
			Error::SteamProton(format!("Library not found for Steam app {app_id}"))
		})?;

		Ok(library
			.path()
			.join("steamapps")
			.join("compatdata")
			.join(app_id.to_string())
			.join("pfx"))
	}

	fn get_wine_binary_path(&self, game: &DbGame) -> Result<PathBuf> {
		let prefix_path = self.get_wine_prefix_path(game)?;
		let compat_data_path = prefix_path.try_parent()?;
		let config_info_path = compat_data_path.join("config_info");

		let config_info_data = fs::read_to_string(&config_info_path)?;

		let proton_lib_path_line = match config_info_data.lines().nth(2) {
			Some(line) if !line.trim().is_empty() => line.trim(),
			_ => {
				return Err(Error::SteamProton(
					"Steam Proton config_info is missing a valid third line".to_string(),
				));
			}
		};

		Ok(Path::new(proton_lib_path_line)
			.try_parent()?
			.join("bin")
			.join("wine"))
	}

	fn run_with_wine(
		&self,
		game: &DbGame,
		exe_path: &Path,
		args: &[String],
		wine_env: &BTreeMap<String, String>,
	) -> Result {
		let wine_prefix_path = self.get_wine_prefix_path(game)?;

		let wine_binary_path = self.get_wine_binary_path(game)?;

		let compat_data_path = wine_prefix_path.try_parent()?;

		let child = Command::new(&wine_binary_path)
			.env("WINEPREFIX", &wine_prefix_path)
			.env("STEAM_COMPAT_DATA_PATH", compat_data_path)
			.env("WINEFSYNC", "1")
			.envs(wine_env)
			.arg(exe_path)
			.args(args)
			.spawn()?;

		log::info!(
			"Launched `{}` with Wine `{}` for Steam game `{}` ({}) using prefix `{}` (pid {})",
			exe_path.display(),
			wine_binary_path.display(),
			game.display_title,
			game.external_id,
			wine_prefix_path.display(),
			child.id(),
		);

		Ok(())
	}

	fn set_wine_dll_overrides(&self, game: &DbGame, dll_overrides: &[String]) -> Result {
		let path = self.get_wine_prefix_path(game)?.join("user.reg");

		if !path.exists() {
			return Err(Error::SteamProton(
				"Steam Proton user.reg doesn't exist yet for this game".to_string(),
			));
		}

		let user_reg_data = fs::read_to_string(&path)?;
		let mut ensured_user_reg_data = user_reg_data.clone();

		for dll_override in dll_overrides {
			let normalized_name = normalize_dll_override_name(dll_override);
			ensured_user_reg_data = reg_add_in_section(
				&ensured_user_reg_data,
				DLL_OVERRIDES_SECTION,
				&normalized_name,
				DLL_OVERRIDE_VALUE,
			);
		}

		if user_reg_data != ensured_user_reg_data {
			let backup_path = path.parent().map_or_else(
				|| path.with_extension("reg.bak"),
				|parent| parent.join("user.reg.bak"),
			);

			fs::copy(&path, backup_path)?;
			fs::write(&path, ensured_user_reg_data)?;

			log::info!("Updated Steam Proton user.reg at {}", path.display());
		}

		Ok(())
	}
}

fn normalize_dll_override_name(dll_name: &str) -> String {
	if dll_name.len() > 4 && dll_name.to_ascii_lowercase().ends_with(".dll") {
		dll_name[..dll_name.len() - 4].to_string()
	} else {
		dll_name.to_string()
	}
}

fn reg_add_in_section(reg_data: &str, section: &str, key: &str, value: &str) -> String {
	let newline = if reg_data.contains("\r\n") {
		"\r\n"
	} else {
		"\n"
	};

	let mut lines = if reg_data.is_empty() {
		Vec::new()
	} else {
		reg_data
			.split(newline)
			.map(std::string::ToString::to_string)
			.collect::<Vec<_>>()
	};

	let key_prefix = format!("\"{key}\"=");
	let key_value_line = format!("\"{key}\"=\"{value}\"");

	if let Some(section_start) = lines
		.iter()
		.position(|line| line.trim_start().starts_with(section))
	{
		let section_end_candidate = lines
			.iter()
			.enumerate()
			.skip(section_start + 1)
			.find(|(_, line)| line.trim_start().starts_with('['));

		let section_end = if let Some((index, _)) = section_end_candidate {
			index
		} else {
			lines.len()
		};

		if let Some(existing_key_index) = lines
			.iter()
			.enumerate()
			.skip(section_start + 1)
			.take(section_end.saturating_sub(section_start + 1))
			.find(|(_, line)| line.trim_start().starts_with(&key_prefix))
			.map(|(index, _)| index)
		{
			lines[existing_key_index] = key_value_line;
		} else {
			lines.insert(section_end, key_value_line);
		}
	} else {
		if !lines.is_empty() && !lines.last().is_some_and(String::is_empty) {
			lines.push(String::new());
		}

		lines.push(section.to_string());
		lines.push(key_value_line);
	}

	let mut updated = lines.join(newline);
	if !updated.ends_with(newline) {
		updated.push_str(newline);
	}

	updated
}
