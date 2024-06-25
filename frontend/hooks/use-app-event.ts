import { events } from "@api/bindings";
import { useEffect, useRef } from "react";

type AppEvents = typeof events;
export type AppEvent = AppEvents[keyof AppEvents];

export function useAppEvent<TEvent extends AppEvent>(
	event: TEvent,
	callback: (
		payload: Parameters<Parameters<TEvent["listen"]>[0]>[0]["payload"],
	) => void,
) {
	const callbackRef = useRef(callback);

	useEffect(() => {
		const unlistenPromise = event.listen((event) =>
			callbackRef.current(event.payload),
		);

		return () => {
			unlistenPromise.then((unlisten) => unlisten());
		};
	}, [event]);
}
