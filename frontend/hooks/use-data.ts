import { useEffect } from "react";
import { atom, useSetAtom } from "jotai";
import { commands, events } from "@api/bindings";
import { dataSubscription } from "./use-data-subscription";
import { useUpdateData } from "./use-update-data";
import { useAsyncCommand } from "./use-async-command";
import { dataPartialSubscription } from "./use-data-partial-subscription";
import { useAppEvent } from "./use-app-event";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import { Store } from "@tauri-apps/plugin-store";

const { addGame } = commands;

export const [installedGamesAtom, useInstalledGamesSubscription] =
	dataPartialSubscription("foundInstalledGame", (payload) => payload.id);

export const [ownedGamesAtom, useOwnedGamesSubscription] =
	dataPartialSubscription("foundOwnedGame", (payload) => payload.id);

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

export const [remoteGamesAtom, useRemoteGamesSubscription] = dataSubscription(
	events.syncRemoteGames,
	[],
);

export const loadingAtom = atom<boolean>(false);

export const dataCacheStore = new Store(".data-cache.dat");

export function useData() {
	useInstalledGamesSubscription();
	useRemoteGamesSubscription();
	useModLoadersSubscription();
	useLocalModsSubscription();
	useRemoteModsSubscription();
	useOwnedGamesSubscription();
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
