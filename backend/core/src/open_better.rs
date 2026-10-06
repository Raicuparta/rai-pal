use std::{
	ffi::OsStr,
	process::{Command, Stdio},
};

use crate::{
	path_extensions::AsValidStr,
	result::{Error, Result},
};

// Bunch of fucking hacks to avoid a bunch of fucking things.
pub fn spawn_detached(cmd: &mut Command) -> Result {
	cmd.env_remove("LD_LIBRARY_PATH")
		.env_remove("QT_PLUGIN_PATH")
		.env_remove("APPDIR")
		.env_remove("APPIMAGE");

	#[cfg(unix)]
	{
		use std::os::unix::process::CommandExt;

		cmd.stdin(Stdio::null())
			.stdout(Stdio::null())
			.stderr(Stdio::null());

		// SAFETY: the closure runs in the forked child right before exec, and
		// only calls async-signal-safe libc functions (setsid, fork, _exit).
		unsafe {
			cmd.pre_exec(|| {
				// New session: the child becomes its own session and process
				// group leader, detached from rai-pal's session and terminal.
				if libc::setsid() == -1 {
					return Err(std::io::Error::last_os_error());
				}

				// Double-fork: the intermediate exits right away, and the
				// grandchild gets reparented to init, so the launched program
				// is never a direct child of rai-pal.
				match libc::fork() {
					-1 => Err(std::io::Error::last_os_error()),
					// Grandchild: continue on to exec the program.
					0 => Ok(()),
					// Intermediate: exit immediately, never returns.
					_ => libc::_exit(0),
				}
			});
		}

		// The direct child is only the intermediate process, which exits a
		// moment after forking. Reap it so it doesn't linger as a zombie.
		let mut child = cmd.spawn()?;
		child.wait()?;
	}

	#[cfg(windows)]
	{
		// Console apps need a console to run in; detaching them makes them run
		// invisibly with I/O redirected to null, so spawn those normally.
		if crate::game_engines::pe_utils::is_pe_console_app(std::path::Path::new(cmd.get_program()))
		{
			cmd.spawn()?;
		} else {
			use std::os::windows::process::CommandExt;

			const DETACHED_PROCESS: u32 = 0x0000_0008;
			const CREATE_NEW_PROCESS_GROUP: u32 = 0x0000_0200;

			cmd.stdin(Stdio::null())
				.stdout(Stdio::null())
				.stderr(Stdio::null())
				.creation_flags(DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP);
			cmd.spawn()?;
		}
	}

	Ok(())
}

// Weird workaround for AppImage builds.
pub fn open_detached_better(path: impl AsRef<OsStr>) -> Result {
	let mut last_error = Error::NoCommandForOpen(path.as_ref().try_to_str()?.to_string());

	for mut cmd in open::commands(path) {
		cmd.env_remove("LD_LIBRARY_PATH");
		cmd.env_remove("QT_PLUGIN_PATH");
		cmd.env_remove("APPDIR");
		cmd.env_remove("APPIMAGE");

		match spawn_detached(&mut cmd) {
			Ok(()) => return Ok(()),
			Err(e) => last_error = e,
		}
	}

	Err(last_error)
}
