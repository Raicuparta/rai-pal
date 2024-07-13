import { useSetAtom, atom } from "jotai";
import { AppEvent, EventPayload, useAppEvent } from "./use-app-event";
import { useRef, useEffect } from "react";

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

	let dataBuffer: EventPayload<TEvent>[] = [];

	stateAtom.write;

	function useDataSubscription() {
		const setData = useSetAtom(stateAtom);
		const dataRef = useRef(defaultValue);
		const flushTimeout = useRef<number | undefined>();

		useAppEvent(event, (payload) => {
			if (payload) {
				dataBuffer.push(payload);
				scheduleFlush();
			}
		});

		const scheduleFlush = useRef(() => {
			if (!flushTimeout.current) {
				flushTimeout.current = setTimeout(() => {
					for (const bufferItem of dataBuffer) {
						dataRef.current.set(getId(bufferItem), bufferItem);
					}
					dataBuffer = [];
					setData({
						data: dataRef.current,
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
