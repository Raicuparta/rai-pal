// Build scripts aren't shipped with the app, so the stricter runtime lints
// around filesystem errors and panics don't apply here.
#![allow(clippy::disallowed_methods, clippy::unwrap_used)]

use std::{
	fs,
	path::{Path, PathBuf},
};

fn emit_translation_reruns(directory: &Path) {
	let entries = match fs::read_dir(directory) {
		Ok(entries) => entries,
		Err(err) => {
			println!(
				"cargo:warning=Failed to read translation directory {}: {err}",
				directory.display()
			);
			return;
		}
	};

	for entry in entries.flatten() {
		let path = entry.path();
		if path.is_dir() {
			emit_translation_reruns(&path);
		} else {
			println!("cargo:rerun-if-changed={}", path.display());
		}
	}
}

fn main() {
	let translations = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("translations");
	emit_translation_reruns(&translations);

	let config = slint_build::CompilerConfiguration::new().with_bundled_translations(translations);

	slint_build::compile_with_config("ui/main_window.slint", config).unwrap();
}
