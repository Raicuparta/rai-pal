import { useEffect } from "react";
import { event } from "@tauri-apps/api";
import { atom } from "jotai";
import { commands, events } from "@api/bindings";
import { dataSubscription } from "./use-data-subscription";
import { useUpdateData } from "./use-update-data";
import { useAsyncCommand } from "./use-async-command";
import { dataPartialSubscription } from "./use-data-partial-subscription";

const { addGame } = commands;

export const [installedGamesAtom, useInstalledGamesSubscription] =
	dataPartialSubscription(
		events.foundInstalledGame,
		(payload) => payload.id,
		{},
	);

export const [modLoadersAtom, useModLoadersSubscription] = dataSubscription(
	events.syncModLoaders,
	{},
);

export const [localModsAtom, useLocalModsSubscription] = dataSubscription(
	events.syncLocalMods,
	{},
);

export const [remoteModsAtom, useRemoteModsSubscription] = dataSubscription(
	events.syncRemoteMods,
	{},
);

export const [ownedGamesAtom, useOwnedGamesSubscription] =
	dataPartialSubscription(events.foundOwnedGame, (payload) => payload.id, {});

export const [remoteGamesAtom, useRemoteGameDataSubscription] =
	dataPartialSubscription(events.foundRemoteGame, (payload) => payload.id, {});

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
