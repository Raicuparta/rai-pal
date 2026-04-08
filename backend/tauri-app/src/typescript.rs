use specta_typescript::Typescript;

const BINDINGS_PATH: &str = "../../frontend/api/bindings.ts";

pub fn export(builder: &tauri_specta::Builder<tauri::Wry>) {
	builder
		.export(Typescript::default(), BINDINGS_PATH)
		.unwrap_or_else(|err| {
			log::error!("Failed to generate TypeScript bindings: {err}");
			std::process::exit(1);
		});
}
