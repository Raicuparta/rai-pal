import { useSetAtom, atom } from "jotai";
import { AppEventId, EventPayload, useAppEvent } from "./use-app-event";
import { useCallback } from "react";

export function dataSubscription<TEventId extends AppEventId>(
	eventId: TEventId,
	defaultValue: EventPayload<TEventId>,
) {
	const stateAtom = atom<EventPayload<TEventId>>(defaultValue);

	function useDataSubscription() {
		const setData = useSetAtom(stateAtom);
		const eventCallback = useCallback(
			(payload: EventPayload<TEventId>) => {
				console.log("useDataSubscription", eventId, payload);
				setData(payload);
			},
			[setData],
		);

		useAppEvent(eventId, eventCallback);
	}

	return [stateAtom, useDataSubscription] as const;
}
