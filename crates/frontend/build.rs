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

fn check_ui_diagnostics() {
	let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui/main_window.slint");
	let source = fs::read_to_string(&path).expect("failed to read the UI source");
	let mut diag = i_slint_compiler::diagnostics::BuildDiagnostics::default();
	let syntax_node = i_slint_compiler::parser::parse(
		source,
		Some(i_slint_compiler::source_path::SourcePath::new(&path)),
		&mut diag,
	);
	let config = i_slint_compiler::CompilerConfiguration::new(
		i_slint_compiler::generator::OutputFormat::Rust,
	);

	let (_, diag, _) = spin_on::spin_on(i_slint_compiler::compile_syntax_node(
		syntax_node,
		diag,
		config,
	));

	if diag.is_empty() {
		return;
	}

	diag.print();
	std::process::exit(1);
}

fn main() {
	println!("cargo:rerun-if-env-changed=RAI_PAL_CHECK_UI_DIAGNOSTICS");
	if std::env::var_os("RAI_PAL_CHECK_UI_DIAGNOSTICS").is_some() {
		check_ui_diagnostics();
	}

	let translations = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("translations");
	emit_translation_reruns(&translations);

	let config = slint_build::CompilerConfiguration::new().with_bundled_translations(translations);

	slint_build::compile_with_config("ui/main_window.slint", config).unwrap();

	embed_windows_icon();
}
