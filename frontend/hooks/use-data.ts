import { atom } from "jotai";
import { commands, events, InstalledGame, OwnedGame } from "@api/bindings";
import { dataSubscription } from "./use-data-subscription";
import { useUpdateData } from "./use-update-data";
import { useGameDropEvent } from "./use-game-drop-event";
import { commandData } from "./use-command-data";

export const installedGamesAtom = atom<Record<string, InstalledGame>>({});
export const ownedGamesAtom = atom<Record<string, OwnedGame>>({});

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
	useRemoteGamesSubscription();
	useModLoadersSubscription();
	useLocalModsSubscription();
	useRemoteModsSubscription();
	useGameDropEvent();
	useUpdateData(true);
}
