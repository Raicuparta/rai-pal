import { useEffect, useState } from "react";

export function usePersistedState<TState>(key: string, defaultState: TState) {
	const [state, setState] = useState<TState>(() => {
		try {
			const persistedState = localStorage.getItem(key);
			return persistedState ? JSON.parse(persistedState) : defaultState;
		} catch (error) {
			console.log(
				`Failed to get localStorage state with key "${key}": ${error}`,
			);
			return defaultState;
		}
	});

	useEffect(() => {
		if (state) {
			localStorage.setItem(key, JSON.stringify(state));
		} else {
			localStorage.removeItem(key);
		}
	}, [state, key]);

	return [state, setState] as const;
}
