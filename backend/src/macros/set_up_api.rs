#[macro_export]
macro_rules! set_up_api {
	( $builder:expr, [ $($func:ident),* $(,)? ] ) => {
		// (
			$builder.invoke_handler(tauri::generate_handler![$($func),*])
			// ,
			// specta::collect_types![$($func),*]
		// )
	};
}
