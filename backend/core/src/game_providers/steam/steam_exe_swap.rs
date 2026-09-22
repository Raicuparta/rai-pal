use std::{
	collections::BTreeMap,
	ffi::OsStr,
	fs,
	os::unix::fs::PermissionsExt,
	path::{Path, PathBuf},
};

use crate::{
	game::DbGame,
	open_better::open_detached_better,
	path_extensions::AsValidStr,
	result::{Error, Result},
};

const BACKUP_SUFFIX: &str = ".rai-pal-orig";
const SCRIPT_PREFIX: &str = "#!/bin/sh\n";

fn backup_path(exe_path: &Path) -> Result<PathBuf> {
	let file_name = exe_path
		.file_name()
		.and_then(OsStr::to_str)
		.ok_or_else(|| Error::InvalidOsStr(exe_path.display().to_string()))?;

	Ok(exe_path.with_file_name(format!("{file_name}{BACKUP_SUFFIX}")))
}

fn shell_single_quote(value: &str) -> String {
	format!("'{}'", value.replace('\'', "'\\''"))
}

fn shell_quote(value: &OsStr) -> Result<String> {
	let value = value.try_to_str()?;
	Ok(shell_single_quote(value))
}

/// Quotes a value for `export`, but allows `${VAR}` references to the process
/// environment so environment values can *prepend* to variables already set (for
/// example `LD_PRELOAD=libdoorstop.so:${LD_PRELOAD}`).
fn shell_quote_with_env(value: &str) -> String {
	let mut output = String::new();
	let mut rest = value;

	while let Some(start) = rest.find("${") {
		let (literal, after) = rest.split_at(start);
		output.push_str(&shell_single_quote(literal));

		let Some(end) = after.find('}') else {
			output.push_str(&shell_single_quote(after));
			return output;
		};

		let name = &after[2..end];
		if name.is_empty()
			|| !name
				.chars()
				.all(|character| character.is_ascii_alphanumeric() || character == '_')
		{
			output.push_str(&shell_single_quote(&after[..=end]));
			rest = &after[end + 1..];
			continue;
		}

		output.push_str(&format!("\"${{{name}}}\""));
		rest = &after[end + 1..];
	}

	output.push_str(&shell_single_quote(rest));
	output
}

fn build_script(
	exe_path: &Path,
	backup: &Path,
	program: &Path,
	args: &[String],
	environment: &BTreeMap<String, String>,
	cwd: Option<&Path>,
) -> Result<String> {
	let exe_quoted = shell_quote(exe_path.as_os_str())?;
	let mut script = String::from(SCRIPT_PREFIX);

	script.push_str(&format!("rm -f -- {exe_quoted}\n"));
	script.push_str(&format!(
		"mv -f -- {} {exe_quoted}\n",
		shell_quote(backup.as_os_str())?
	));

	for (key, value) in environment {
		script.push_str(&format!("export {key}={}\n", shell_quote_with_env(value)));
	}

	if let Some(cwd) = cwd {
		script.push_str(&format!("cd {} || exit 1\n", shell_quote(cwd.as_os_str())?));
	}

	script.push_str("exec ");
	script.push_str(&shell_quote(program.as_os_str())?);
	for arg in args {
		script.push(' ');
		script.push_str(&shell_quote(OsStr::new(arg))?);
	}
	script.push_str(" \"$@\"\n");

	Ok(script)
}

/// Replaces a native Linux Steam game's executable with a script that restores
/// the real executable, sets the game's environment, and execs `program`, then
/// asks Steam to launch the app.
///
/// Steam only tracks games it launched itself, so launching the game ourselves
/// means Steam's "Exit Game" has nothing to stop. Going through Steam's own
/// launch path (with our script in place of the game binary) gives Steam a
/// launch record, so the whole process tree can be stopped again.
pub fn launch_via_steam_with_swapped_exe(
	game: &DbGame,
	program: &Path,
	args: &[String],
	environment: &BTreeMap<String, String>,
	cwd: Option<&Path>,
) -> Result {
	let exe_path = game.try_get_exe_path()?;
	let backup = backup_path(exe_path)?;

	// Recover from an interrupted previous swap (launch never happened).
	if backup.exists() {
		log::warn!(
			"Found leftover Rai Pal executable backup at `{}`, restoring it before swapping again",
			backup.display()
		);
		let _ = fs::remove_file(exe_path);
		fs::rename(&backup, exe_path)?;
	}

	if !program.exists() {
		return Err(Error::GameNotInstalled(format!(
			"Launch target `{}` does not exist",
			program.display()
		)));
	}

	let script = build_script(exe_path, &backup, program, args, environment, cwd)?;

	fs::rename(exe_path, &backup)?;
	if let Err(error) = fs::write(exe_path, script.as_bytes()) {
		let _ = fs::rename(&backup, exe_path);
		return Err(error.into());
	}
	fs::set_permissions(exe_path, fs::Permissions::from_mode(0o755))?;

	let url = format!("steam://rungameid/{}", game.external_id);
	log::info!(
		"Replaced `{}` with a temporary launcher, asking Steam to launch `{url}`",
		exe_path.display()
	);

	if let Err(error) = open_detached_better(&url) {
		let _ = fs::remove_file(exe_path);
		let _ = fs::rename(&backup, exe_path);
		return Err(error);
	}

	Ok(())
}
