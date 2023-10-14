import { useEffect } from "react";
import { atom, useSetAtom } from "jotai";
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
export const errorAtom = atom<string>("");
export const loadingAtom = atom<boolean>(false);

export function useAppStoreEffect() {
	useListen("SyncInstalledGames", installedGamesAtom, getInstalledGames);
	useListen("SyncOwnedGames", ownedGamesAtom, getOwnedGames);
	useListen("SyncDiscoverGames", discoverGamesAtom, getDiscoverGames);
	useListen("SyncMods", modLoadersAtom, getModLoaders);
	const setError = useSetAtom(errorAtom);
	const setIsLoading = useSetAtom(loadingAtom);

	useEffect(() => {
		setIsLoading(true);
		updateState()
			.catch(setError)
			.finally(() => setIsLoading(false));
	}, [setError, setIsLoading]);
}
