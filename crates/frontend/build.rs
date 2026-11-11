// Build scripts aren't shipped with the app, so the stricter runtime lints
// around filesystem errors and panics don't apply here.
#![allow(clippy::disallowed_methods, clippy::expect_used, clippy::unwrap_used)]

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

fn embed_windows_icon() {
	let icon = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("graphics/icon.ico");
	println!("cargo:rerun-if-changed={}", icon.display());

	let out_dir = std::env::var_os("OUT_DIR").expect("OUT_DIR is set by Cargo");
	let resource_script = PathBuf::from(out_dir).join("icon.rc");
	let escaped_icon = icon.to_string_lossy().replace('\\', "\\\\");
	fs::write(&resource_script, format!("1 ICON \"{escaped_icon}\"\n"))
		.expect("failed to write resource script");

	embed_resource::compile(&resource_script, embed_resource::NONE)
		.manifest_optional()
		.expect("failed to embed Windows icon");
}

fn main() {
	let translations = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("translations");
	emit_translation_reruns(&translations);

	let config = slint_build::CompilerConfiguration::new().with_bundled_translations(translations);

	slint_build::compile_with_config("ui/main_window.slint", config).unwrap();

	embed_windows_icon();
}
