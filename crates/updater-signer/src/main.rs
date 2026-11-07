use std::{env, fs::File, path::PathBuf, process::ExitCode};

use base64::{Engine as _, engine::general_purpose::STANDARD};
use minisign::SecretKeyBox;

fn main() -> ExitCode {
	match run() {
		Ok(()) => ExitCode::SUCCESS,
		Err(err) => {
			eprintln!("{err}");
			ExitCode::FAILURE
		}
	}
}

fn run() -> Result<(), String> {
	let path = env::args()
		.nth(1)
		.map(PathBuf::from)
		.ok_or_else(|| "Usage: rai-pal-updater-signer <file>".to_string())?;

	let key =
		env::var("UPDATE_PRIVATE_KEY").map_err(|_| "Missing UPDATE_PRIVATE_KEY".to_string())?;
	let password = env::var("UPDATE_KEY_PASSWORD")
		.ok()
		.filter(|password| !password.is_empty());

	let key = match STANDARD.decode(key.trim()) {
		Ok(bytes) => String::from_utf8(bytes).map_err(|err| err.to_string())?,
		Err(_) => key,
	};

	let make_box = || SecretKeyBox::from_string(&key).map_err(|err| err.to_string());
	let secret_key = if let Some(password) = password {
		match minisign::SecretKey::from_box(make_box()?, Some(password)) {
			Ok(secret_key) => secret_key,
			Err(_) => minisign::SecretKey::from_unencrypted_box(make_box()?)
				.map_err(|err| err.to_string())?,
		}
	} else {
		minisign::SecretKey::from_unencrypted_box(make_box()?).map_err(|err| err.to_string())?
	};
	let file = File::open(&path).map_err(|err| err.to_string())?;
	let signature =
		minisign::sign(None, &secret_key, file, None, None).map_err(|err| err.to_string())?;

	print!("{}", STANDARD.encode(signature.into_string()));
	Ok(())
}
