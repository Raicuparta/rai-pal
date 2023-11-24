import { AppEvent } from "@api/bindings";
import { useSetAtom, atom } from "jotai";
import { useAppEvent } from "./use-app-event";
import { Result } from "@api/result";

export function dataSubscription<TData, TError>(
	event: AppEvent,
	apiFunction: () => Promise<Result<TData, TError>>,
	defaultValue: TData,
) {
	const stateAtom = atom<TData>(defaultValue);

	function useDataSubscription() {
		const setData = useSetAtom(stateAtom);

		useAppEvent(event, () => {
			apiFunction().then((data) => {
				// TODO handle error
				if (data.status === "ok") {
					setData(data.data);
				}
			});
		});
	}

	return [stateAtom, useDataSubscription] as const;
}
