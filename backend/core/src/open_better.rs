use std::{
	ffi::OsStr,
	process::{
		Command,
		Stdio,
	},
};

use crate::{
	path_extensions::AsValidStr,
	result::{
		Error,
		Result,
	},
};

/// Spawns `cmd` fully detached from rai-pal, so that games, mod scripts, and
/// other launched programs:
///
/// - Are **not killed** when rai-pal is killed (or when its terminal closes
///   or its process group receives a signal).
/// - Are **not left behind as children of rai-pal** once it exits.
/// - Never linger as zombie processes under a running rai-pal.
///
/// How it works per platform:
///
/// - **Unix** (`setsid` + double-fork via [`Command::pre_exec`]): the child
///   first becomes the leader of a brand new session and process group, so it
///   no longer shares rai-pal's session/terminal/process group. Then it forks
///   once more: the intermediate process exits immediately (and is reaped
///   here), while the actual program keeps running as a grandchild that gets
///   reparented to init. It is therefore not a direct child of rai-pal at
///   all, and init reaps it when it eventually exits.
/// - **Windows** (`DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP`): the child
///   gets its own console and process group, so console close / Ctrl+C and
///   process-group signals don't reach it.
///
/// In all cases stdin/stdout/stderr are redirected to null so the child
/// doesn't hold on to rai-pal's console.
pub fn spawn_detached(cmd: &mut Command) -> Result {
	cmd.stdin(Stdio::null())
		.stdout(Stdio::null())
		.stderr(Stdio::null());

	#[cfg(unix)]
	{
		use std::os::unix::process::CommandExt;

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
		use std::os::windows::process::CommandExt;

		const DETACHED_PROCESS: u32 = 0x0000_0008;
		const CREATE_NEW_PROCESS_GROUP: u32 = 0x0000_0200;

		cmd.creation_flags(DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP);
		cmd.spawn()?;
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
