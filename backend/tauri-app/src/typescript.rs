use specta_typescript::Typescript;

const BINDINGS_PATH: &str = "../../frontend/api/bindings.ts";

pub fn export(builder: &tauri_specta::Builder<tauri::Wry>) {
	#[allow(
		clippy::unwrap_used,
		reason = "This is dev only, we want it to explode if bindings fail to update."
	)]
	builder
		.export(Typescript::default(), BINDINGS_PATH)
		.unwrap();
}
