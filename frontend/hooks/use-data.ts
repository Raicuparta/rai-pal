import { atom } from "jotai";
import { GameIdsResponse } from "@api/bindings";
import { dataSubscription } from "./use-data-subscription";
import { useUpdateData } from "./use-update-data";
import { useGameDropEvent } from "./use-game-drop-event";

export const gameDataAtom = atom<GameIdsResponse>({
	gameIds: [],
	totalCount: 0,
});

export const [modsAtom, useModsSubscription] = dataSubscription("syncMods", {});

type LoadingTask = {
	index: number;
	name: string;
};

export const loadingTasksAtom = atom<LoadingTask[]>([]);

export function useData() {
	useModsSubscription();
	useGameDropEvent();
	useUpdateData(true);
}
