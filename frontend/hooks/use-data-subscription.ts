import { AppEvent, Error, Result } from "@api/bindings";
import { useSetAtom, atom } from "jotai";
import { useAppEvent } from "./use-app-event";

export function dataSubscription<TData>(
	event: AppEvent,
	apiFunction: () => Promise<Result<TData, Error>>,
	defaultValue: TData,
) {
	const stateAtom = atom<TData>(defaultValue);

	function useDataSubscription() {
		const setData = useSetAtom(stateAtom);

		useAppEvent(event, () => {
			apiFunction().then((result) => {
				if (result.status == "ok") {
					setData(result.data);
				} else {
					// TODO: handle error
					console.error(result.error);
				}
			});
		});
	}

	return [stateAtom, useDataSubscription] as const;
}
