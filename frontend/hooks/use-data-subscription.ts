import { Error, Result } from "@api/bindings";
import { useSetAtom, atom } from "jotai";
import { AppEvent, EventPayload, useAppEvent } from "./use-app-event";

export function dataSubscription<TEvent extends AppEvent>(
	event: TEvent,
	defaultValue: EventPayload<TEvent>,
) {
	const stateAtom = atom<EventPayload<TEvent>>(defaultValue);

	function useDataSubscription() {
		const setData = useSetAtom(stateAtom);

		useAppEvent(event, setData);
	}

	return [stateAtom, useDataSubscription] as const;
}
