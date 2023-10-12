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
	updateState: () => void;
	clearError: () => void;
};

export const useAppStore = create<State & Actions>((setStore, getStore) => ({
	isReady: false,
	error: "",
	isLoading: false,
	data: {
		discoverGames: [],
		gameMap: {},
		modLoaders: {},
		ownedGames: [],
	},

	clearError: () => setStore({ error: "" }),

	updateState: () => {
		setStore({ isLoading: true, error: "" });

		getFullState()
			.then((data) => setStore({ data, isReady: true }))
			.catch((error) => setStore({ error }))
			.finally(() => setStore({ isLoading: false }));
	},
}));

export function useAppStoreEffect() {
	const updateAppStore = useAppStore((store) => store.updateState);

	useEffect(() => {
		updateAppStore();
	}, [updateAppStore]);
}
