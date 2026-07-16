use specta_typescript::Typescript;

const BINDINGS_PATH: &str = concat!(
	env!("CARGO_MANIFEST_DIR"),
	"/../../frontend/api/bindings.ts"
);

pub fn export(builder: &tauri_specta::Builder<tauri::Wry>) {
	#[allow(
		clippy::unwrap_used,
		reason = "This is dev only, we wanna make sure it explodes if types fail to generate."
	)]
	builder
		.export(Typescript::default(), BINDINGS_PATH)
		.unwrap();
}
