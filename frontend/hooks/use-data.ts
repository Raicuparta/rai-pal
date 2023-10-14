import { useEffect } from "react";
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
		updateData();
	}, [updateData]);
}
