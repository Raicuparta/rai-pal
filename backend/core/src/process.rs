use tokio::process::{Child, Command};

use crate::{
	game::DbGame, mods::game_mod::PreparedModRun, path_extensions::PathExt, result::Result,
};

#[cfg(target_os = "linux")]
use crate::{
	game_engines::pe_utils, game_providers::game_provider, operating_system::OperatingSystem,
	result::Error,
};

#[cfg(windows)]
use std::process::Command as SyncCommand;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

/// Spawns a managed mod process, owned by Rai Pal.
///
/// On Unix the child leads its own process group, so its descendants can be
/// killed together with it. On Windows it gets no console window.
pub fn spawn_managed(run: &PreparedModRun, game_option: Option<&DbGame>) -> Result<Child> {
	let mut command = build_command(run, game_option)?;

	command
		.current_dir(run.path.try_parent()?)
		.kill_on_drop(true);

	#[cfg(unix)]
	command.process_group(0);
	#[cfg(windows)]
	command.creation_flags(CREATE_NO_WINDOW);

	Ok(command.spawn()?)
}

#[cfg_attr(
	target_os = "windows",
	expect(unused_variables, reason = "Only used for Wine on Linux")
)]
fn build_command(run: &PreparedModRun, game_option: Option<&DbGame>) -> Result<Command> {
	#[cfg(target_os = "linux")]
	if run.os == Some(OperatingSystem::Windows) {
		let game = game_option.ok_or_else(Error::GameNeeded)?;
		let provider = game_provider::get_provider(game.provider_id)?;
		let mut command = provider.get_run_with_wine_command(game)?;

		if pe_utils::is_pe_console_app(&run.path) {
			command.arg("wineconsole");
		}

		command
			.arg(&run.path)
			.args(&run.args)
			.envs(&run.wine_environment);

		return Ok(Command::from(command));
	}

	let mut command = Command::new(&run.path);
	command.args(&run.args);
	Ok(command)
}

/// Sends a termination signal to a managed process and its descendants.
///
/// When `force` is false this is a graceful request (SIGTERM, or `taskkill`
/// without `/F`). When it's true, the process is killed forcefully. Processes
/// that already exited are ignored.
pub fn terminate_managed_process(pid: u32, force: bool) -> Result {
	#[cfg(unix)]
	{
		let signal = if force { libc::SIGKILL } else { libc::SIGTERM };

		// SAFETY: killpg is async-signal-safe and only reads its arguments.
		unsafe {
			libc::killpg(pid.cast_signed(), signal);
		}
	}

	#[cfg(windows)]
	{
		let mut command = SyncCommand::new("taskkill");
		command.args(["/PID", &pid.to_string(), "/T"]);

		if force {
			command.arg("/F");
		}

		let _ = command.status();
	}

	Ok(())
}
