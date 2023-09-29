#[macro_export]
macro_rules! set_up_tauri {
    ($types_output_path:expr, $app_state:expr, [ $($func:ident),* ]) => {
        if let Err(specta_err) = specta::collect_types![$($func),*].map(|types| {
            #[cfg(debug_assertions)]
            return tauri_specta::ts::export_with_cfg(
                types,
                specta::ts::ExportConfiguration::default().bigint(specta::ts::BigIntExportBehavior::BigInt),
                $types_output_path,
            );
        }) {
            println!("Failed to generate TypeScript bindings: {specta_err}");
        }

        tauri::Builder::default()
            .manage($app_state)
            .invoke_handler(tauri::generate_handler![$($func),*])
            .run(tauri::generate_context!())
            .unwrap_or_else(|err| println!("Failed to run Tauri application: {err}"));
    };
}
