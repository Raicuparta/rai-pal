#[macro_export]
macro_rules! set_up_api {
	( $builder:expr, [ $($func:ident),* $(,)? ] ) => {
		#[cfg(debug_assertions)]
		{
			let types = specta::collect_types![
					$($func),*
			];
			let result = types.map(|types| {
					return tauri_specta::ts::export_with_cfg(
							types,
							specta::ts::ExportConfiguration::default()
									.bigint(specta::ts::BigIntExportBehavior::BigInt),
							"../frontend/api/bindings.ts",
					);
			});

			if let Err(specta_err) = result {
				println!("Failed to generate TypeScript bindings: {}", specta_err);
			}
		}

		$builder.invoke_handler(tauri::generate_handler![$($func),*])
			.run(tauri::generate_context!())
			.unwrap_or_else(|err| println!("Failed to run Tauri application: {err}"));
	};
}
