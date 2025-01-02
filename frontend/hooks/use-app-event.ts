import { events } from "@api/bindings";
import { useEffect } from "react";

type AppEvents = typeof events;
type EventCallback<TEventId extends AppEventId> = (
	payload: EventPayload<TEventId>,
) => void;
export type AppEventId = keyof AppEvents;
export type AppEvent<TEventId extends AppEventId = AppEventId> =
	AppEvents[TEventId];
export type EventPayload<TEventId extends AppEventId> = Parameters<
	Parameters<AppEvent<TEventId>["listen"]>[0]
>[0]["payload"];

// Apparently, calling Tauri's `listen` very frequently (like many times per second)
// can cause the frontend to slow to a crawl. So we need to cache these listeners.
const eventCallbacks: Map<
	AppEventId,
	Array<EventCallback<AppEventId>>
> = new Map();

function getCallbacks(eventId: AppEventId) {
	let callbacks = eventCallbacks.get(eventId);
	if (!callbacks) {
		callbacks = [];
		eventCallbacks.set(eventId, callbacks);
		events[eventId].listen((eventObject) => {
			for (const callback of eventCallbacks.get(eventId) ?? []) {
				callback(eventObject.payload);
			}
		});
	}
	return callbacks;
}

export function useAppEvent<TEventId extends AppEventId>(
	eventId: TEventId,
	callback: EventCallback<TEventId>,
) {
	useEffect(() => {
		const callbacks = getCallbacks(eventId);
		const index = callbacks.push(callback);

		return () => {
			callbacks.splice(index - 1, 1);
		};
	}, [eventId, callback]);
}
