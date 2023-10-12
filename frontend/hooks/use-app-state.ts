import { FullState, getFullState } from "@api/bindings";
// import { atom, useAtom } from "jotai";
import { create } from "zustand";
import { useCallback, useEffect } from "react";

type State = {
	isReady: boolean;
	error: string;
	isLoading: boolean;
	data: FullState;
};

type Actions = {
	updateState: (state: Partial<State>) => void;
};

export const useAppStore = create<State & Actions>((set) => ({
	isReady: false,
	error: "",
	isLoading: false,
	data: {
		discoverGames: [],
		gameMap: {},
		modLoaders: {},
		ownedGames: [],
	},
	updateState: (newState: Partial<State>) =>
		set((previousState) => ({ ...previousState, ...newState })),
}));

export function useAppState() {
	const updateAppState = useAppStore((store) => store.updateState);
	const isReady = useAppStore((store) => store.isReady);

	const clearError = useCallback(() => updateAppState({ error: "" }), []);

	const updateData = useCallback(
		(ignoreCache = false) => {
			if (!ignoreCache && isReady) return;

			updateAppState({ isLoading: true });
			clearError();

			getFullState()
				.then((data) => updateAppState({ data, isReady: true }))
				.catch((error) => updateAppState({ error }))
				.finally(() => updateAppState({ isLoading: false }));
		},
		[clearError],
	);

	const refresh = useCallback(() => {
		updateData(true);
	}, [updateData]);

	useEffect(() => {
		updateData();
	}, [updateData]);

	// return [appState, isLoading, refresh, error, clearError] as const;
}
