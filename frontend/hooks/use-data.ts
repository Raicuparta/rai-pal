import { atom } from "jotai";
import { GameIdsResponse, GameMod, ModFamily } from "@api/bindings";
import { useUpdateData } from "./use-update-data";
import { useGameDropEvent } from "./use-game-drop-event";

export const gameDataAtom = atom<GameIdsResponse>({
	gameIds: [],
	totalCount: 0,
});

export const modsAtom = atom<Record<string, GameMod>>({});

export const modFamiliesAtom = atom<Record<string, ModFamily>>({});

export const gameDataVersionAtom = atom(0);

export function useData() {
	useGameDropEvent();
	useUpdateData(true);
}
