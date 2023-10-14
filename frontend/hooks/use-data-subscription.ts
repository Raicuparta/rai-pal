import { SyncDataEvent } from "@api/bindings";
import { listen } from "@tauri-apps/api/event";
import { PrimitiveAtom, useSetAtom } from "jotai";
import { useEffect } from "react";

type Atom<TData> = PrimitiveAtom<TData>;

export function useDataSubscription<TData>(
	event: SyncDataEvent,
	stateAtom: Atom<TData>,
	apiFunction: () => Promise<TData>,
) {
	const setData = useSetAtom(stateAtom);

	useEffect(() => {
		let unlisten: Awaited<ReturnType<typeof listen>> | undefined;

		(async () => {
			unlisten = await listen(event, () =>
				apiFunction()
					.then(setData)
					.finally(() => console.log("called event", event)),
			);
		})();

		return () => {
			if (unlisten) unlisten();
		};
	}, [apiFunction, event, setData]);
}
