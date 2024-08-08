import { atom } from "jotai";
import { commands, events } from "@api/bindings";
import { dataSubscription } from "./use-data-subscription";
import { useUpdateData } from "./use-update-data";
import { dataPartialSubscription } from "./use-data-partial-subscription";
import { useGameDropEvent } from "./use-game-drop-event";
import { commandData } from "./use-command-data";

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

export const [remoteGamesAtom, useRemoteGamesSubscription] = commandData(
	commands.fetchRemoteGames,
	[],
);

export const loadingCountAtom = atom(0);

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
