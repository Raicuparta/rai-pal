import { atom } from "jotai";
import { commands, GameData } from "@api/bindings";
import { dataSubscription } from "./use-data-subscription";
import { useUpdateData } from "./use-update-data";
import { useGameDropEvent } from "./use-game-drop-event";
import { commandData } from "./use-command-data";

export const gameDataAtom = atom<GameData>({
	gameIds: [],
	totalCount: BigInt(0),
});

export const [modLoadersAtom, useModLoadersSubscription] = dataSubscription(
	"syncModLoaders",
	{},
);

export const [localModsAtom, useLocalModsSubscription] = dataSubscription(
	"syncLocalMods",
	{},
);

export const [remoteModsAtom, useRemoteModsSubscription] = dataSubscription(
	"syncRemoteMods",
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
