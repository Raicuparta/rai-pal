import { SyncDataEvent } from "@api/bindings";
import { listen } from "@tauri-apps/api/event";
import { useSetAtom, atom } from "jotai";
import { useEffect } from "react";

export function dataSubscription<TData>(
	event: SyncDataEvent,
	apiFunction: () => Promise<TData>,
	defaultValue: TData,
) {
	const stateAtom = atom<TData>(defaultValue);

	function useDataSubscription() {
		const setData = useSetAtom(stateAtom);

		useEffect(() => {
			let unlisten: Awaited<ReturnType<typeof listen>> | undefined;

			(async () => {
				unlisten = await listen(event, () =>
					apiFunction()
						.then((data) => setData(data as TData))
						.finally(() => console.log("called event", event)),
				);
			})();

			return () => {
				if (unlisten) unlisten();
			};
		}, [setData]);
	}

	return [stateAtom, useDataSubscription] as const;
}
