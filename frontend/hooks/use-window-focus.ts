import { getCurrentWindow } from "@tauri-apps/api/window";
import { useEffect, useEffectEvent } from "react";

export function useWindowFocus(callback: () => void) {
	const onFocus = useEffectEvent(callback);

	useEffect(() => {
		const unlisten = getCurrentWindow().onFocusChanged(
			({ payload: focused }) => {
				if (focused) onFocus();
			},
		);

		return () => {
			unlisten.then((removeListener) => removeListener());
		};
	}, []);
}
