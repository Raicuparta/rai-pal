import { useEffect } from "react";
import { atom, useSetAtom } from "jotai";
import { commands, events } from "@api/bindings";
import { dataSubscription } from "./use-data-subscription";
import { useUpdateData } from "./use-update-data";
import { useAsyncCommand } from "./use-async-command";
import { dataPartialSubscription } from "./use-data-partial-subscription";
import { useAppEvent } from "./use-app-event";
import { getCurrentWebview } from "@tauri-apps/api/webview";

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

	useAppEvent(events.gameRemoved, (gameId) => {
		setInstalledGames((previousInstalledGames) => {
			// Mutating the inner value because we're doing some cursed thing for performance.
			previousInstalledGames.data.delete(gameId);

			// But creating a new object for the outer structure, to count as a new reference.
			return {
				...previousInstalledGames,
			};
		});
	});

	useEffect(() => {
		const unlistenPromise = getCurrentWebview().onDragDropEvent((event) => {
			if (event.payload.type === "drop") {
				executeAddGame(event.payload.paths[0]);
			}
		});

		return () => {
			unlistenPromise.then((unlisten) => unlisten());
		};
	}, [executeAddGame]);

	useEffect(() => {
		updateData();
	}, [updateData]);
}
