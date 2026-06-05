import { atom } from "jotai";
import { GameIdsResponse, GameMod } from "@api/bindings";
import { useUpdateData } from "./use-update-data";
import { useGameDropEvent } from "./use-game-drop-event";

export const gameDataAtom = atom<GameIdsResponse>({
	gameIds: [],
	totalCount: 0,
});

export const modsAtom = atom<Record<string, GameMod>>({});

type LoadingTask = {
	index: number;
	name: string;
};

export const loadingTasksAtom = atom<LoadingTask[]>([]);

export function useData() {
	useGameDropEvent();
	useUpdateData(true);
}
