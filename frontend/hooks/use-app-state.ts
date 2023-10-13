import { FullState, getFullState } from "@api/bindings";
import { listen } from "@tauri-apps/api/event";
// import { atom, useAtom } from "jotai";
import { create } from "zustand";
import { useEffect } from "react";

type State = {
	isReady: boolean;
	error: string;
	isLoading: boolean;
	data: FullState;
};

type Actions = {
	updateState: () => void;
	setData: (data: FullState) => void;
	clearError: () => void;
};

export const useAppStore = create<State & Actions>((setStore) => ({
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

	setData: (data) => {
		const cleanData = Object.fromEntries(
			Object.entries(data).filter(([, value]) => value),
		);

		setStore((store) => ({ ...store, data: { ...store.data, ...cleanData } }));
	},

	updateState: () => {
		setStore({ isLoading: true, error: "" });

		getFullState()
			.then((data) => setStore({ data }))
			.catch((error) => setStore({ error }))
			.finally(() => setStore({ isLoading: false }));
	},
}));

export function useAppStoreEffect() {
	const updateState = useAppStore((store) => store.updateState);
	const setData = useAppStore((store) => store.setData);

	useEffect(() => {
		updateState();

		let unlisten: Awaited<ReturnType<typeof listen>> | undefined;

		(async () => {
			unlisten = await listen("update_state", updateState);
		})();

		return () => {
			if (unlisten) unlisten();
		};
	}, [setData, updateState]);
}
