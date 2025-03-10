import { useSetAtom, atom } from "jotai";
import { AppEventId, EventPayload, useAppEvent } from "./use-app-event";

export function dataSubscription<TEventId extends AppEventId>(
	eventId: TEventId,
	defaultValue: EventPayload<TEventId>,
) {
	const stateAtom = atom<EventPayload<TEventId>>(defaultValue);

	function useDataSubscription() {
		const setData = useSetAtom(stateAtom);
		const eventCallback = (payload: EventPayload<TEventId>) => {
			setData(payload);
		};

		useAppEvent(eventId, "data-subscription", eventCallback);
	}

	return [stateAtom, useDataSubscription] as const;
}
