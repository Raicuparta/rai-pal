use std::{
	collections::BTreeMap,
	ffi::{OsStr, OsString},
	path::PathBuf,
	process::Command,
};

#[cfg(target_os = "linux")]
use std::{
	process::Stdio,
	sync::atomic::{AtomicU64, Ordering},
};

use crate::{open_better::spawn_detached, result::Result};

/// A game command with everything needed to spawn it independently of Rai Pal.
///
/// We can't use a `std::process::Command` for this because Steam tracks a game
/// by the process tree rooted at the `reaper` it spawns, and a child of Rai Pal
/// stays inside that tree even after `setsid`/double-fork. To escape it we ask
/// systemd to spawn the game as a separate user service, which needs the program,
/// arguments and environment separately.
#[derive(Clone, Default)]
pub struct GameLaunch {
	pub program: PathBuf,
	pub args: Vec<OsString>,
	pub env: BTreeMap<String, String>,
	pub cwd: Option<PathBuf>,
}

impl GameLaunch {
	pub fn new(program: impl Into<PathBuf>) -> Self {
		Self {
			program: program.into(),
			..Default::default()
		}
	}

	pub fn arg(&mut self, arg: impl Into<OsString>) -> &mut Self {
		self.args.push(arg.into());
		self
	}

	pub fn args<I, S>(&mut self, args: I) -> &mut Self
	where
		I: IntoIterator<Item = S>,
		S: Into<OsString>,
	{
		self.args.extend(args.into_iter().map(Into::into));
		self
	}

	pub fn env(&mut self, key: impl Into<String>, value: impl AsRef<OsStr>) -> &mut Self {
		self.env
			.insert(key.into(), value.as_ref().to_string_lossy().into_owned());
		self
	}

	pub fn envs(&mut self, env: &BTreeMap<String, String>) -> &mut Self {
		self.env
			.extend(env.iter().map(|(key, value)| (key.clone(), value.clone())));
		self
	}

	/// Like [`Self::envs`], but expands `${VAR}` references in values against the
	/// environment set so far. Shell-based launches (the Steam exe swap) get this
	/// for free; direct spawns need it so values like
	/// `libdoorstop.so:${LD_PRELOAD}` don't end up literal.
	pub fn envs_expanded(&mut self, env: &BTreeMap<String, String>) -> &mut Self {
		for (key, value) in env {
			let value = expand_environment_variables(value, &self.env);
			self.env.insert(key.clone(), value);
		}
		self
	}

	pub fn with_cwd(mut self, cwd: impl Into<PathBuf>) -> Self {
		self.cwd = Some(cwd.into());
		self
	}

	pub fn to_command(&self) -> Command {
		let mut command = Command::new(&self.program);
		command.args(&self.args).envs(&self.env);

		if let Some(cwd) = &self.cwd {
			command.current_dir(cwd);
		}

		command
	}
}

#[cfg(target_os = "linux")]
static SYSTEMD_SERVICE_COUNTER: AtomicU64 = AtomicU64::new(0);

fn expand_environment_variables(value: &str, environment: &BTreeMap<String, String>) -> String {
	let mut output = String::new();
	let mut rest = value;

	while let Some(start) = rest.find("${") {
		output.push_str(&rest[..start]);

		let after = &rest[start + 2..];
		let Some(end) = after.find('}') else {
			output.push_str("${");
			rest = after;
			continue;
		};

		let name = &after[..end];
		if name.is_empty()
			|| !name
				.chars()
				.all(|character| character.is_ascii_alphanumeric() || character == '_')
		{
			output.push_str("${");
			rest = after;
			continue;
		}

		if let Some(replacement) = environment.get(name) {
			output.push_str(replacement);
		}

		rest = &after[end + 1..];
	}

	output.push_str(rest);
	output
}

#[cfg(target_os = "linux")]
const ENV_VARS_TO_KEEP_OUT: &[&str] = &[
	"LD_LIBRARY_PATH",
	"LD_PRELOAD",
	"QT_PLUGIN_PATH",
	"APPDIR",
	"APPIMAGE",
	"INVOCATION_ID",
	"JOURNAL_STREAM",
	"LISTEN_FDS",
	"LISTEN_PID",
];

#[cfg(target_os = "linux")]
fn find_in_path(name: &str) -> Option<PathBuf> {
	let path = std::env::var_os("PATH")?;
	std::env::split_paths(&path)
		.map(|dir| dir.join(name))
		.find(|candidate| candidate.is_file())
}

#[cfg(target_os = "linux")]
fn has_user_systemd() -> bool {
	std::env::var_os("XDG_RUNTIME_DIR")
		.map(PathBuf::from)
		.is_some_and(|runtime_dir| runtime_dir.join("systemd").exists())
}

/// Builds a `systemd-run` invocation that asks the user systemd manager to spawn
/// the game as a transient service. Unlike a direct child of Rai Pal, that service
/// is parented to the systemd manager, so Steam's `reaper` does not consider it
/// part of Rai Pal's process tree.
#[cfg(target_os = "linux")]
fn systemd_run_command(launch: &GameLaunch) -> Option<Command> {
	if !has_user_systemd() {
		return None;
	}

	let systemd_run = find_in_path("systemd-run")?;

	let unit = format!(
		"rai-pal-game-{}-{}",
		std::process::id(),
		SYSTEMD_SERVICE_COUNTER.fetch_add(1, Ordering::Relaxed)
	);

	let mut command = Command::new(systemd_run);
	command
		.arg("--user")
		.arg("--collect")
		.arg(format!("--unit={unit}"))
		.arg("--quiet")
		.stdin(Stdio::null())
		.stdout(Stdio::null())
		.stderr(Stdio::null());

	if let Some(cwd) = &launch.cwd {
		command.arg(format!("--working-directory={}", cwd.display()));
	}

	let mut env: BTreeMap<String, String> = std::env::vars()
		.filter(|(key, _)| !ENV_VARS_TO_KEEP_OUT.contains(&key.as_str()))
		.collect();
	env.extend(
		launch
			.env
			.iter()
			.map(|(key, value)| (key.clone(), value.clone())),
	);

	for (key, value) in &env {
		command.arg(format!("--setenv={key}={value}"));
	}

	command.arg("--").arg(&launch.program).args(&launch.args);

	Some(command)
}

/// Spawns a game outside of Rai Pal's process tree, so that closing Rai Pal (and
/// Steam tearing it down) does not kill the game. Falls back to a plain detached
/// spawn where systemd is not available.
pub fn spawn_game(launch: &GameLaunch) -> Result {
	#[cfg(target_os = "linux")]
	if let Some(mut command) = systemd_run_command(launch) {
		let status = command.status()?;

		if status.success() {
			log::info!(
				"Started game through systemd user service: `{}`",
				launch.program.display()
			);

			return Ok(());
		}

		log::warn!(
			"systemd-run failed to start the game, falling back to a detached spawn: `{}`",
			launch.program.display()
		);
	}

	let mut command = launch.to_command();
	spawn_detached(&mut command)
}
