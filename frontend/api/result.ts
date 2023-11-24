// TODO Result should be exported by tauri-specta
export type Result<TData, TError> =
	| { status: "ok"; data: TData }
	| { status: "error"; error: TError };
