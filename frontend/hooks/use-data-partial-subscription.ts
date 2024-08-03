import { useSetAtom, atom } from "jotai";
import {
	AppEvent,
	AppEventId,
	EventPayload,
	useAppEvent,
} from "./use-app-event";
import { useEffect, useRef } from "react";
import { useThrottledCallback } from "@mantine/hooks";
import { events } from "@api/bindings";
import { dataCacheStore } from "./use-data";

type PartialData<TEvent extends AppEvent> = {
	data: Map<string, EventPayload<TEvent>>;
};

export function dataPartialSubscription<
	TEventId extends AppEventId,
	TEvent extends AppEvent<TEventId>,
	TData extends Map<string, EventPayload<TEvent>>,
>(eventId: TEventId, getId: (payload: EventPayload<TEvent>) => string) {
	const cacheId = `data-partial-subscription-cache-${eventId}`;

	const defaultData = new Map() as TData;
	const stateAtom = atom<PartialData<TEvent>>({
		data: defaultData,
	});

	function useDataSubscription() {
		const setData = useSetAtom(stateAtom);
		const dataRef = useRef(defaultData);
		const updateState = useThrottledCallback(() => {
			setData({
				data: dataRef.current,
			});
			dataCacheStore.set(cacheId, [...dataRef.current]);
		}, 500);

		useAppEvent(events[eventId], (payload) => {
			dataRef.current.set(getId(payload), payload);
			updateState();
		});

		useEffect(() => {
			try {
				dataCacheStore
					.get<[string, EventPayload<TEvent>][]>(cacheId)
					.then((data) => {
						if (data) {
							dataRef.current = new Map([...data, ...dataRef.current]) as TData;
						}
					});
			} catch (error) {
				console.error("Failed to load cached data. Clearing cache", error);
				dataCacheStore.set(cacheId, []);
			}
		}, []);

		return stateAtom;
	}

	return [stateAtom, useDataSubscription] as const;
}
