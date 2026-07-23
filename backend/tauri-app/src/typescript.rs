use std::io::Read;

use specta_typescript::Typescript;

const BINDINGS_PATH: &str = concat!(
	env!("CARGO_MANIFEST_DIR"),
	"/../../frontend/api/bindings.ts"
);

pub fn export(builder: &tauri_specta::Builder<tauri::Wry>) {
	#[expect(
		clippy::unwrap_used,
		reason = "This is dev only, we wanna make sure it explodes if types fail to generate."
	)]
	builder
		.export(Typescript::default(), BINDINGS_PATH)
		.unwrap();

	// Specta generates `{ [key in T]: V }` without constraining T, which causes TS errors.
	let mut content = String::new();
	#[expect(
		clippy::unwrap_used,
		reason = "This is dev only, we wanna make sure it explodes if types fail to generate."
	)]
	std::fs::File::open(BINDINGS_PATH)
		.unwrap()
		.read_to_string(&mut content)
		.unwrap();
	#[expect(
		clippy::unwrap_used,
		reason = "This is dev only, we wanna make sure it explodes if types fail to generate."
	)]
	std::fs::write(BINDINGS_PATH, format!("// @ts-nocheck\n{content}")).unwrap();
}
