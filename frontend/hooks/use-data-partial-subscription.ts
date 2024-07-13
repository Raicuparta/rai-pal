import { useSetAtom, atom } from "jotai";
import { AppEvent, EventPayload, useAppEvent } from "./use-app-event";
import { useRef } from "react";
import { useThrottledCallback } from "@mantine/hooks";

type PartialData<TEvent extends AppEvent> = {
	data: Map<string, EventPayload<TEvent>>;
};

export function dataPartialSubscription<TEvent extends AppEvent>(
	event: TEvent,
	getId: (payload: EventPayload<TEvent>) => string,
	defaultValue: Map<string, EventPayload<TEvent>>,
) {
	const stateAtom = atom<PartialData<TEvent>>({
		data: defaultValue,
	});

	function useDataSubscription() {
		const setData = useSetAtom(stateAtom);
		const dataRef = useRef(defaultValue);
		const updateState = useThrottledCallback(() => {
			setData({
				data: dataRef.current,
			});
		}, 500);

		useAppEvent(event, (payload) => {
			dataRef.current.set(getId(payload), payload);
			updateState();
		});

		return stateAtom;
	}

	return [stateAtom, useDataSubscription] as const;
}
