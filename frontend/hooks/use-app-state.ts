import {
	LocalState,
	RemoteState,
	getLocalState,
	getRemoteState,
} from "@api/bindings";
import { listen } from "@tauri-apps/api/event";
import { create } from "zustand";
import { useEffect } from "react";

type State = {
	isReady: boolean;
	error: string;
	isLoading: boolean;
	localState: LocalState;
	remoteState: RemoteState;
};

type Actions = {
	updateLocal: () => void;
	updateRemote: () => void;
	setData: (data: {
		localState?: LocalState;
		remoteState?: RemoteState;
	}) => void;
	clearError: () => void;
};

export const useAppStore = create<State & Actions>((setStore) => ({
	isReady: false,
	error: "",
	isLoading: false,
	localState: {
		gameMap: {},
		modLoaders: {},
	},
	remoteState: {
		discoverGames: [],
		ownedGames: [],
	},

	clearError: () => setStore({ error: "" }),

	setData: (data) => {
		const cleanData = Object.fromEntries(
			Object.entries(data).filter(([, value]) => value),
		);

		setStore((store) => ({ ...store, ...cleanData }));
	},

	updateLocal: () => {
		setStore({ isLoading: true, error: "" });

		getLocalState()
			.then((data) => setStore({ localState: data }))
			.catch((error) => setStore({ error }))
			.finally(() => setStore({ isLoading: false }));
	},

	updateRemote: () => {
		setStore({ isLoading: true, error: "" });

		getRemoteState()
			.then((data) => setStore({ remoteState: data }))
			.catch((error) => setStore({ error }))
			.finally(() => setStore({ isLoading: false }));
	},
}));

export function useAppStoreEffect() {
	const updateLocal = useAppStore((store) => store.updateLocal);
	const updateRemote = useAppStore((store) => store.updateRemote);
	const setData = useAppStore((store) => store.setData);

	useEffect(() => {
		updateLocal();
		updateRemote();

		let unlisten: Awaited<ReturnType<typeof listen>> | undefined;

		(async () => {
			unlisten = await listen("sync_local", updateLocal);
		})();

		return () => {
			if (unlisten) unlisten();
		};
	}, [setData, updateLocal, updateRemote]);
}
