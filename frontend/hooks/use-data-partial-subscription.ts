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
import { getDataCache } from "../util/data-cache";

type PartialData<TEvent extends AppEvent> = {
	data: Record<string, EventPayload<TEvent>>;
};

export function dataPartialSubscription<
	TEventId extends AppEventId,
	TEvent extends AppEvent<TEventId>,
>(eventId: TEventId, getId: (payload: EventPayload<TEvent>) => string) {
	type DataRecord = Record<string, EventPayload<TEvent>>;

	const cacheId = `data-partial-subscription-cache-${eventId}`;
	const defaultData = {} as DataRecord;
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
			getDataCache().set(cacheId, dataRef.current);
		}, 500);

		useAppEvent(events[eventId], (payload) => {
			dataRef.current[getId(payload)] = payload;
			updateState();
		});

		useEffect(() => {
			getDataCache()
				.get<DataRecord>(cacheId)
				.then((data) => {
					if (data) {
						dataRef.current = { ...data, ...dataRef.current };
						updateState();
					}
				})
				.catch((error) => {
					console.error("Failed to load cached data. Clearing cache", error);
					getDataCache().set(cacheId, []);
				});
		}, [updateState]);

		return stateAtom;
	}

	return [stateAtom, useDataSubscription] as const;
}
