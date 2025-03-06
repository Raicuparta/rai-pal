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

type LoadingTask = {
	index: number;
	name: string;
};

export const loadingTasksAtom = atom<LoadingTask[]>([]);

export function useData() {
	useModLoadersSubscription();
	useLocalModsSubscription();
	useRemoteModsSubscription();
	useGameDropEvent();
	useUpdateData(true);
}
