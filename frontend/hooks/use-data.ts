import { useEffect } from "react";
import { event } from "@tauri-apps/api";
import { atom } from "jotai";
import { commands } from "@api/bindings";
import { dataSubscription } from "./use-data-subscription";
import { useUpdateData } from "./use-update-data";
import { useAsyncCommand } from "./use-async-command";

const {
	getInstalledGames,
	getModLoaders,
	getOwnedGames,
	getLocalMods,
	getRemoteMods,
	addGame,
	getRemoteGames,
} = commands;

export const [installedGamesAtom, useInstalledGamesSubscription] =
	dataSubscription("SyncInstalledGames", getInstalledGames, {});

export const [modLoadersAtom, useModLoadersSubscription] = dataSubscription(
	"SyncModLoaders",
	getModLoaders,
	{},
);

export const [localModsAtom, useLocalModsSubscription] = dataSubscription(
	"SyncLocalMods",
	getLocalMods,
	{},
);

export const [remoteModsAtom, useRemoteModsSubscription] = dataSubscription(
	"SyncRemoteMods",
	getRemoteMods,
	{},
);

export const [ownedGamesAtom, useOwnedGamesSubscription] = dataSubscription(
	"SyncOwnedGames",
	getOwnedGames,
	{},
);

export const [remoteGamesAtom, useRemoteGameDataSubscription] =
	dataSubscription("SyncRemoteGames", getRemoteGames, {});

export const loadingAtom = atom<boolean>(false);

export function useData() {
	useInstalledGamesSubscription();
	useModLoadersSubscription();
	useLocalModsSubscription();
	useRemoteModsSubscription();
	useOwnedGamesSubscription();
	useRemoteGameDataSubscription();

	const updateData = useUpdateData();

	const [executeAddGame] = useAsyncCommand(addGame);

	useEffect(() => {
		const unlistenPromise = event.listen<string[]>(
			event.TauriEvent.DROP,
			(event) => {
				if (event.payload.length > 0) {
					executeAddGame(event.payload[0]);
				}
			},
		);

		return () => {
			unlistenPromise.then((unlisten) => unlisten());
		};
	}, [executeAddGame]);

	useEffect(() => {
		updateData();
	}, [updateData]);
}
