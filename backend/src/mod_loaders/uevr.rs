use std::{
	ffi::OsStr,
	io,
	os::windows::ffi::OsStrExt,
	path::Path,
	ptr,
};

use winapi::{
	ctypes::c_int,
	um::{
		shellapi::ShellExecuteW,
		winuser::SW_SHOW,
	},
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
		let exe_path = self.data.path.join(Self::EXE_NAME);
		let exe_path_str: Vec<u16> = OsStr::new(&exe_path).encode_wide().chain(Some(0)).collect();

		let verb = OsStr::new("runas");
		let verb_str: Vec<u16> = verb.encode_wide().chain(Some(0)).collect();

		let parameters = format!(
			"--attach=\"{}\"",
			get_actual_unreal_binary(&game.full_path)
				.file_name()
				.ok_or_else(|| Error::FailedToGetFileName(game.full_path.clone()))?
				.to_string_lossy()
		);
		let parameters_str: Vec<u16> = OsStr::new(&parameters)
			.encode_wide()
			.chain(Some(0))
			.collect();

		let directory = self.data.path.as_os_str();
		let directory_str: Vec<u16> = directory.encode_wide().chain(Some(0)).collect();

		let result = unsafe {
			ShellExecuteW(
				ptr::null_mut(),
				verb_str.as_ptr(),
				exe_path_str.as_ptr(),
				parameters_str.as_ptr(),
				directory_str.as_ptr(),
				SW_SHOW,
				// SEE_MASK_NOASYNC | SEE_MASK_FLAG_NO_UI,
			)
		};

		#[allow(clippy::as_conversions)]
		if result as c_int > 32 {
			Ok(())
		} else {
			Err(io::Error::last_os_error())?
		}
	}

	fn open_mod_folder(&self, _mod_id: &str) -> Result {
		Ok(open::that_detached(&self.data.path)?)
	}
}
