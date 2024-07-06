import { useSetAtom, atom } from "jotai";
import { AppEvent, EventPayload, useAppEvent } from "./use-app-event";
import { useRef, useEffect } from "react";

export function dataPartialSubscription<TEvent extends AppEvent>(
	event: TEvent,
	getId: (payload: EventPayload<TEvent>) => string,
	defaultValue: Map<string, EventPayload<TEvent>>,
) {
	const stateAtom = atom<Map<string, EventPayload<TEvent>>>(defaultValue);
	const dataMap = new Map<string, EventPayload<TEvent>>();

	stateAtom.write;

	function useDataSubscription() {
		const setData = useSetAtom(stateAtom);
		const flushTimeout = useRef<number | undefined>();

		useAppEvent(event, (payload) => {
			if (payload) {
				dataMap.set(getId(payload), payload);
				scheduleFlush();
			}
		});

		const scheduleFlush = useRef(() => {
			if (!flushTimeout.current) {
				flushTimeout.current = setTimeout(() => {
					setData((previousData) => {
						for (const [key, value] of dataMap) {
							previousData.set(key, value);
						}
						dataMap.clear();
						return new Map(previousData);
					});
					flushTimeout.current = undefined;
				}, 500);
			}
		}).current;

		useEffect(() => {
			return () => {
				if (flushTimeout.current) {
					clearTimeout(flushTimeout.current);
				}
			};
		}, []);

		return stateAtom;
	}

	return [stateAtom, useDataSubscription] as const;
}
