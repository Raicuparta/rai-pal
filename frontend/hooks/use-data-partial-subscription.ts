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

	let accumulatedMapTime = 0;
	let accumulatedStateTime = 0;

	function useDataSubscription() {
		const setData = useSetAtom(stateAtom);
		const dataRef = useRef(defaultValue);
		const updateState = useThrottledCallback(() => {
			setData({
				data: dataRef.current,
			});
		}, 1000);

		useAppEvent(event, (payload) => {
			let startTime = performance.now();
			dataRef.current.set(getId(payload), payload);
			accumulatedMapTime += performance.now() - startTime;
			startTime = performance.now();
			updateState();
			accumulatedStateTime += performance.now() - startTime;
		});

		setInterval(() => {
			console.log(`Map time: ${accumulatedMapTime}ms`);
			console.log(`State time: ${accumulatedStateTime}ms`);
		}, 5000);

		return stateAtom;
	}

	return [stateAtom, useDataSubscription] as const;
}
