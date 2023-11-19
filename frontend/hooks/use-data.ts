import { useEffect } from "react";
import { addGame } from "@api/bindings";
import { event } from "@tauri-apps/api";
import { atom } from "jotai";
import {
	getDiscoverGames,
	getInstalledGames,
	getModLoaders,
	getOwnedGames,
} from "@api/bindings";
import { dataSubscription } from "./use-data-subscription";
import { useUpdateData } from "./use-update-data";

export const [installedGamesAtom, useInstalledGamesSubscription] =
	dataSubscription("SyncInstalledGames", getInstalledGames, {});

export const [discoverGamesAtom, useDiscoverGamesSubscription] =
	dataSubscription("SyncDiscoverGames", getDiscoverGames, []);

export const [modLoadersAtom, useModLoadersSubscription] = dataSubscription(
	"SyncMods",
	getModLoaders,
	{},
);
export const [ownedGamesAtom, useOwnedGamesSubscription] = dataSubscription(
	"SyncOwnedGames",
	getOwnedGames,
	[],
);

export const errorAtom = atom<string>("");
export const loadingAtom = atom<boolean>(false);

export function useData() {
	useInstalledGamesSubscription();
	useDiscoverGamesSubscription();
	useModLoadersSubscription();
	useOwnedGamesSubscription();

	const updateData = useUpdateData();

	useEffect(() => {
		const unlistenPromise = event.listen<string[]>(
			event.TauriEvent.WINDOW_FILE_DROP,
			(event) => {
				if (event.payload.length > 0) {
					addGame(event.payload[0]);
					updateData();
				}
			},
		);

		return () => {
			unlistenPromise.then((unlisten) => unlisten());
		};
	}, [updateData]);

	useEffect(() => {
		updateData();
	}, [updateData]);
}
