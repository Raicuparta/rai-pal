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

type CallbackMap = Map<string, EventCallback<AppEventId>>;

// Apparently, calling Tauri's `listen` very frequently (like many times per second)
// can cause the frontend to slow to a crawl. So we need to cache these listeners.
const eventCallbacks: Map<AppEventId, CallbackMap> = new Map();

function getCallbacks(eventId: AppEventId) {
	let callbacks: CallbackMap | undefined = eventCallbacks.get(eventId);
	if (!callbacks) {
		callbacks = new Map();
		eventCallbacks.set(eventId, callbacks);
		events[eventId].listen((eventObject) => {
			for (const callback of eventCallbacks.get(eventId)?.values() ?? []) {
				callback(eventObject.payload);
			}
		});
	}
	return callbacks;
}

export function useAppEvent<TEventId extends AppEventId>(
	eventId: TEventId,
	key: string,
	callback: EventCallback<TEventId>,
) {
	useEffect(() => {
		const callbacks = getCallbacks(eventId);
		callbacks.set(key, callback);

		return () => {
			callbacks.delete(key);
		};
	}, [eventId, callback, key]);
}
