import { useEffect, useState } from "react";

export function usePersistedState<TState>(key: string, defaultState: TState) {
	const [state, setState] = useState<TState>(() => {
		const persistedState = localStorage.getItem(key);
		return persistedState !== null ? JSON.parse(persistedState) : defaultState;
	});

	useEffect(() => {
		localStorage.setItem(key, JSON.stringify(state));
	}, [state, key]);

	return [state, setState] as const;
}
