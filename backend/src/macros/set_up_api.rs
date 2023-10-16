#[macro_export]
macro_rules! set_up_api {
	( $builder:expr, [ $($func:ident),* $(,)? ] ) => {
		{
		$builder.invoke_handler(tauri::generate_handler![$($func),*])
			.run(tauri::generate_context!())
			.unwrap_or_else(|err| println!("Failed to run Tauri application: {err}"));

			specta::collect_types![$($func),*]
		}
	};
}
