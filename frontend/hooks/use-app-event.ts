import { AppEvent } from "@api/bindings";
import { listen } from "@tauri-apps/api/event";
import { useEffect, useRef } from "react";

export function useAppEvent(event: AppEvent, callback: () => void) {
	const callbackRef = useRef(callback);

	useEffect(() => {
		let unlisten: Awaited<ReturnType<typeof listen>> | undefined;

		(async () => {
			unlisten = await listen(event, () => callbackRef.current());
		})();

		return () => {
			if (unlisten) unlisten();
		};
	}, [event]);
}
