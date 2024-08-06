import { useCallback, useEffect, useState } from "react";
import {
	getLocalStorage,
	listenToStorageChange,
	setLocalStorage,
} from "../util/local-storage";

// This uses localStorage to persist state.
// The persisted data will often randomly disappear, but localStorage is really fast to use.
// So this is good for stuff that is convenient to persist, but not a big deal if it gets lost.
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
