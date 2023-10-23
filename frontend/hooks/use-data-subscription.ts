import { AppEvent } from "@api/bindings";
import { useSetAtom, atom } from "jotai";
import { useAppEvent } from "./use-app-event";

export function dataSubscription<TData>(
	event: AppEvent,
	apiFunction: () => Promise<TData>,
	defaultValue: TData,
) {
	const stateAtom = atom<TData>(defaultValue);

	function useDataSubscription() {
		const setData = useSetAtom(stateAtom);

		useAppEvent(event, () => {
			apiFunction().then((data) => setData(data as TData));
		});
	}

	return [stateAtom, useDataSubscription] as const;
}
