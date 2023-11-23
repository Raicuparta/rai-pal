import { AppEvent } from "@api/bindings";
import { listen } from "@tauri-apps/api/event";
import { useEffect, useRef } from "react";

export function useAppEvent<TPayload = void>(
	event: AppEvent,
	callback: (payload: TPayload) => void,
) {
	const callbackRef = useRef(callback);

	useEffect(() => {
		let unlisten: Awaited<ReturnType<typeof listen>> | undefined;

		(async () => {
			unlisten = await listen(event, (event) =>
				callbackRef.current(event.payload as TPayload),
			);
		})();

		return () => {
			if (unlisten) unlisten();
		};
	}, [event]);
}
