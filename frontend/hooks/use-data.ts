import { useCallback, useEffect } from "react";
import { event } from "@tauri-apps/api";
import { atom, useSetAtom } from "jotai";
import { commands, events } from "@api/bindings";
import { dataSubscription } from "./use-data-subscription";
import { useUpdateData } from "./use-update-data";
import { useAsyncCommand } from "./use-async-command";
import { dataPartialSubscription } from "./use-data-partial-subscription";
import { useAppEvent } from "./use-app-event";

const { addGame } = commands;

export const [installedGamesAtom, useInstalledGamesSubscription] =
	dataPartialSubscription(
		events.foundInstalledGame,
		(payload) => payload.id,
		new Map(),
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
	dataPartialSubscription(
		events.foundOwnedGame,
		(payload) => payload.id,
		new Map(),
	);

export const [remoteGamesAtom, useRemoteGameDataSubscription] =
	dataPartialSubscription(
		events.foundRemoteGame,
		(payload) => payload.id,
		new Map(),
	);

export const loadingAtom = atom<boolean>(false);

export function useData() {
	useInstalledGamesSubscription();
	useModLoadersSubscription();
	useLocalModsSubscription();
	useRemoteModsSubscription();
	useOwnedGamesSubscription();
	useRemoteGameDataSubscription();
	const setInstalledGames = useSetAtom(installedGamesAtom);

	const updateData = useUpdateData();

	const [executeAddGame] = useAsyncCommand(addGame);

	const onGameRemoved = useCallback(
		(gameId: string) => {
			setInstalledGames((previousInstalledGames) => {
				// Mutating the inner value because we're doing some cursed thing for performance.
				previousInstalledGames.data.delete(gameId);

				// But creating a new object for the outer structure, to count as a new reference.
				return {
					...previousInstalledGames,
				};
			});
		},
		[setInstalledGames],
	);

	useAppEvent(events.gameRemoved, onGameRemoved);

	useEffect(() => {
		const unlistenPromise = event.listen<string[]>(
			event.TauriEvent.DRAG_DROP,
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
