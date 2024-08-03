import { type events } from "@api/bindings";
import { useCallbackRef } from "@mantine/hooks";
import { useEffect } from "react";

type AppEvents = typeof events;
export type AppEventId = keyof AppEvents;
export type AppEvent<TEventId extends AppEventId = AppEventId> =
	AppEvents[TEventId];
export type EventPayload<TEvent extends AppEvent> = Parameters<
	Parameters<TEvent["listen"]>[0]
>[0]["payload"];

export function useAppEvent<TEvent extends AppEvent>(
	event: TEvent,
	callback: (payload: EventPayload<TEvent>) => void,
) {
	const callbackRef = useCallbackRef(callback);

	useEffect(() => {
		const unlistenPromise = event.listen((event) => callbackRef(event.payload));

		return () => {
			unlistenPromise.then((unlisten) => unlisten());
		};
	}, [event, callbackRef]);
}
