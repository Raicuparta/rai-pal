import { atom } from "jotai";
import { events } from "@api/bindings";
import { dataSubscription } from "./use-data-subscription";
import { useUpdateData } from "./use-update-data";
import { dataPartialSubscription } from "./use-data-partial-subscription";
import { Store } from "@tauri-apps/plugin-store";
import { useGameDropEvent } from "./use-game-drop-event";

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

export const loadingCountAtom = atom(0);

export const dataCacheStore = new Store(".data-cache.dat");

export function useData() {
	useInstalledGamesSubscription();
	useRemoteGamesSubscription();
	useModLoadersSubscription();
	useLocalModsSubscription();
	useRemoteModsSubscription();
	useOwnedGamesSubscription();
	useGameDropEvent();
	useUpdateData(true);
}
