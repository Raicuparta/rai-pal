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
import { useListen } from "./use-listen";
import { useUpdateAppState } from "./use-update-state";

export const installedGamesAtom = atom<Record<string, Game>>({});
export const ownedGamesAtom = atom<OwnedGame[]>([]);
export const discoverGamesAtom = atom<SteamGame[]>([]);
export const modLoadersAtom = atom<Record<string, ModLoaderData>>({});
export const errorAtom = atom<string>("");
export const loadingAtom = atom<boolean>(false);

export function useAppStoreEffect() {
	useListen("SyncInstalledGames", installedGamesAtom, getInstalledGames);
	useListen("SyncOwnedGames", ownedGamesAtom, getOwnedGames);
	useListen("SyncDiscoverGames", discoverGamesAtom, getDiscoverGames);
	useListen("SyncMods", modLoadersAtom, getModLoaders);
	const updateAppState = useUpdateAppState();

	useEffect(() => {
		updateAppState();
	}, [updateAppState]);
}
