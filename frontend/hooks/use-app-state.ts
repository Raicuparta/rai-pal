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
	updateState,
} from "@api/bindings";
import { useListen } from "./use-listen";

export const installedGamesAtom = atom<Record<string, Game>>({});
export const ownedGamesAtom = atom<OwnedGame[]>([]);
export const discoverGamesAtom = atom<SteamGame[]>([]);
export const modLoadersAtom = atom<Record<string, ModLoaderData>>({});

export function useAppStoreEffect() {
	useListen("SyncInstalledGames", installedGamesAtom, getInstalledGames);
	useListen("SyncOwnedGames", ownedGamesAtom, getOwnedGames);
	useListen("SyncDiscoverGames", discoverGamesAtom, getDiscoverGames);
	useListen("SyncMods", modLoadersAtom, getModLoaders);

	useEffect(() => {
		updateState();
	}, []);
}
