import { useCallback, useEffect, useRef, useState } from "react";

const eventIdPrefix = "storage-changed";

export function usePersistedState<TState>(defaultState: TState, key?: string) {
	const eventName = useRef(`${eventIdPrefix}-${key}`);

	const [state, setState] = useState<TState>(() => {
		if (!key) return defaultState;

		try {
			const persistedState = localStorage.getItem(key);
			return persistedState ? JSON.parse(persistedState) : defaultState;
		} catch (error) {
			console.error(
				`Failed to get localStorage state with key "${key}": ${error}`,
			);
			return defaultState;
		}
	});

	useEffect(() => {
		function handleStorageChanged() {
			if (!key) return;

			const persistedState = localStorage.getItem(key);
			setState(persistedState ? JSON.parse(persistedState) : defaultState);
		}

		const currentEventName = eventName.current;

		window.addEventListener(currentEventName, handleStorageChanged);

		return () => {
			window.removeEventListener(currentEventName, handleStorageChanged);
		};
	}, [defaultState, key]);

	const setPersistedState = useCallback(
		(stateSetParam: TState | ((previousState: TState) => TState)) => {
			if (!key) return;

			const newState =
				typeof stateSetParam === "function"
					? (stateSetParam as (newState: TState) => TState)(state)
					: (stateSetParam as TState);

			if (newState) {
				localStorage.setItem(key, JSON.stringify(newState));
			} else {
				localStorage.removeItem(key);
			}

			window.dispatchEvent(new Event(eventName.current));
		},
		[key, state],
	);

	return [state, setPersistedState] as const;
}
