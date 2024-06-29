import { useSetAtom, atom } from "jotai";
import { AppEvent, EventPayload, useAppEvent } from "./use-app-event";

export function dataPartialSubscription<TEvent extends AppEvent>(
	event: TEvent,
	getId: (payload: EventPayload<TEvent>) => string,
	defaultValue: Record<string, EventPayload<TEvent>>,
) {
	const stateAtom = atom<Record<string, EventPayload<TEvent>>>(defaultValue);

	function useDataSubscription() {
		const setData = useSetAtom(stateAtom);

		useAppEvent(event, (payload) =>
			setData((previousData) => {
				if (!payload) return previousData;

				return {
					...previousData,
					[getId(payload)]: payload,
				};
			}),
		);
	}

	return [stateAtom, useDataSubscription] as const;
}
