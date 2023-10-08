use std::{
	path::Path,
	process::Command,
};

use super::mod_loader::{
	ModLoaderActions,
	ModLoaderData,
	ModLoaderStatic,
};
use crate::{
	game::Game,
	game_engines::{
		game_engine::GameEngineBrand,
		unreal::get_actual_unreal_binary,
	},
	game_mod::Mod,
	result::Error,
	serializable_struct,
	Result,
};

serializable_struct!(UeVr {
  pub data: ModLoaderData,
});

impl UeVr {
	const EXE_NAME: &'static str = "UnrealVR.exe";
}

impl ModLoaderStatic for UeVr {
	const ID: &'static str = "uevr";

	fn new(resources_path: &Path) -> Result<Self>
	where
		Self: std::marker::Sized,
	{
		let path = resources_path.join(Self::ID);
		let mods = if path.join(Self::EXE_NAME).exists() {
			vec![Mod::new(
				&resources_path.join(Self::ID),
				Some(GameEngineBrand::Unreal),
				None,
			)?]
		} else {
			vec![]
		};

		Ok(Self {
			data: ModLoaderData {
				id: Self::ID.to_string(),
				mods,
				path,
			},
		})
	}
}

impl ModLoaderActions for UeVr {
	fn get_data(&self) -> &ModLoaderData {
		&self.data
	}

	fn install(&self, _game: &Game) -> Result {
		todo!()
	}

	fn install_mod(&self, game: &Game, _mod_idd: &str) -> Result {
		let command = format!(
			// Messy way to launch the uevr exe, because we need admin privileges.
			"Start-Process -FilePath \"{}\" -ArgumentList '--attach=\"{}\"' -WorkingDirectory \"{}\" -Verb RunAs",
			self.data.path.join(Self::EXE_NAME).to_string_lossy(),
			get_actual_unreal_binary(&game.full_path)
				.file_name()
				.ok_or_else(|| Error::FailedToGetFileName(
					game.full_path.clone()
				))?
				.to_string_lossy(),
			self.data.path.to_string_lossy(),
		);

		Command::new("powershell")
			.arg("-Command")
			.arg(command)
			.spawn()?;

		Ok(())
	}

	fn open_mod_folder(&self, _mod_id: &str) -> Result {
		Ok(open::that_detached(&self.data.path)?)
	}
}
