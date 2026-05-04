import { atom } from "jotai";
import { GameIdsResponse } from "@api/bindings";
import { dataSubscription } from "./use-data-subscription";
import { useUpdateData } from "./use-update-data";
import { useGameDropEvent } from "./use-game-drop-event";

export const gameDataAtom = atom<GameIdsResponse>({
	gameIds: [],
	totalCount: 0,
});

export const [localModLoadersAtom, useLocalModLoadersSubscription] =
	dataSubscription("syncLocalModLoaders", {});

export const [remoteModLoadersAtom, useRemoteModLoadersSubscription] =
	dataSubscription("syncRemoteModLoaders", {});

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
	useLocalModLoadersSubscription();
	useRemoteModLoadersSubscription();
	useLocalModsSubscription();
	useRemoteModsSubscription();
	useGameDropEvent();
	useUpdateData(true);
}
