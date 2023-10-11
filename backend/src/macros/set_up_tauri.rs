#[macro_export]
macro_rules! set_up_tauri {
    ($types_output_path:expr, $app_state:expr, [ $($func:ident),* ]) => {
        let specta_builder = {
            // You can use `tauri_specta::js::builder` for exporting JS Doc instead of Typescript!`
            let specta_builder = tauri_specta::ts::builder()
                .commands(tauri_specta::collect_commands![$($func),*]); // <- Each of your comments


            #[cfg(debug_assertions)] // <- Only export on non-release builds
            let specta_builder = specta_builder.path($types_output_path);

            specta_builder.into_plugin()
        };

        tauri::Builder::default()
            .manage($app_state)
            .plugin(specta_builder)
            .invoke_handler(tauri::generate_handler![$($func),*])
            .run(tauri::generate_context!())
            .unwrap_or_else(|err| println!("Failed to run Tauri application: {err}"));
    };
}
