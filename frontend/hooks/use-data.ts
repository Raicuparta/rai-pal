import { useEffect } from "react";
import { event } from "@tauri-apps/api";
import { atom } from "jotai";
import { commands, events } from "@api/bindings";
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
	dataSubscription(events.syncInstalledGames, getInstalledGames, {});

export const [modLoadersAtom, useModLoadersSubscription] = dataSubscription(
	events.syncModLoaders,
	getModLoaders,
	{},
);

export const [localModsAtom, useLocalModsSubscription] = dataSubscription(
	events.syncLocalMods,
	getLocalMods,
	{},
);

export const [remoteModsAtom, useRemoteModsSubscription] = dataSubscription(
	events.syncRemoteMods,
	getRemoteMods,
	{},
);

export const [ownedGamesAtom, useOwnedGamesSubscription] = dataSubscription(
	events.syncOwnedGames,
	getOwnedGames,
	{},
);

export const [remoteGamesAtom, useRemoteGameDataSubscription] =
	dataSubscription(events.syncRemoteGames, getRemoteGames, {});

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
