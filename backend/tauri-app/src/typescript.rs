use specta_typescript::{BigIntExportBehavior, Typescript};

const BINDINGS_PATH: &str = "../../frontend/api/bindings.ts";

pub fn export(builder: &tauri_specta::Builder<tauri::Wry>) {
	builder
		.export(
			Typescript::default().bigint(BigIntExportBehavior::BigInt),
			BINDINGS_PATH,
		)
		.unwrap_or_else(|err| {
			log::error!("Failed to generate TypeScript bindings: {err}");
			std::process::exit(1);
		});
}
