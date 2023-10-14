import { useEffect } from "react";
import { atom } from "jotai";
import {
	Game,
	ModLoaderData,
	OwnedGame,
	SteamGame,
	getDiscoverGames,
	getInstalledGames,
	getModLoaders,
	getOwnedGames,
} from "@api/bindings";
import { useDataSubscription } from "./use-data-subscription";
import { useUpdateData } from "./use-update-data";

export const installedGamesAtom = atom<Record<string, Game>>({});
export const ownedGamesAtom = atom<OwnedGame[]>([]);
export const discoverGamesAtom = atom<SteamGame[]>([]);
export const modLoadersAtom = atom<Record<string, ModLoaderData>>({});
export const errorAtom = atom<string>("");
export const loadingAtom = atom<boolean>(false);

export function useData() {
	useDataSubscription(
		"SyncInstalledGames",
		installedGamesAtom,
		getInstalledGames,
	);
	useDataSubscription("SyncOwnedGames", ownedGamesAtom, getOwnedGames);
	useDataSubscription("SyncDiscoverGames", discoverGamesAtom, getDiscoverGames);
	useDataSubscription("SyncMods", modLoadersAtom, getModLoaders);
	const updateData = useUpdateData();

	useEffect(() => {
		updateData();
	}, [updateData]);
}
