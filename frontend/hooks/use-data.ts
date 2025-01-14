import { atom } from "jotai";
import { GameIdsResponse } from "@api/bindings";
import { dataSubscription } from "./use-data-subscription";
import { useUpdateData } from "./use-update-data";
import { useGameDropEvent } from "./use-game-drop-event";

export const gameDataAtom = atom<GameIdsResponse>({
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

export const loadingCountAtom = atom(0);

export function useData() {
	useModLoadersSubscription();
	useLocalModsSubscription();
	useRemoteModsSubscription();
	useGameDropEvent();
	useUpdateData(true);
}
