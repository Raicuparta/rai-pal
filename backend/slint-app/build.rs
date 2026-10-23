use std::{collections::HashMap, path::PathBuf};

fn main() {
	let library = HashMap::from([("lucide".to_string(), PathBuf::from(lucide_slint::lib()))]);
	let config = slint_build::CompilerConfiguration::new()
		.with_library_paths(library)
		.with_bundled_translations("translations");

	slint_build::compile_with_config("ui/main-window.slint", config).unwrap();
}
