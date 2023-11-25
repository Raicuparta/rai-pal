import { useCallback, useEffect, useState } from "react";
import {
	getLocalStorage,
	listenToStorageChange,
	setLocalStorage,
} from "../util/local-storage";

export function usePersistedState<TState>(defaultState: TState, key?: string) {
	const [state, setState] = useState<TState>(() => {
		if (!key) return defaultState;

		try {
			return getLocalStorage(key, defaultState);
		} catch (error) {
			console.error(
				`Failed to get localStorage state with key "${key}": ${error}`,
			);
			return defaultState;
		}
	});

	useEffect(() => {
		if (!key) return;

		const unlistenToStorageChange = listenToStorageChange(
			key,
			defaultState,
			setState,
		);

		return () => {
			unlistenToStorageChange();
		};
	}, [defaultState, key]);

	const setPersistedState = useCallback(
		(stateSetParam: TState | ((previousState: TState) => TState)) => {
			if (!key) return;

			const newState =
				typeof stateSetParam === "function"
					? (stateSetParam as (newState: TState) => TState)(state)
					: (stateSetParam as TState);

			setLocalStorage(key, newState);
		},
		[key, state],
	);

	return [state, setPersistedState] as const;
}
