import { useAtomValue } from "jotai";
import { useMemo } from "react";
import { gameIdsAtom } from "./use-data";
import { PageId } from "../pages";

export type TabCounts = Record<PageId, number>;

export function useDataCounts() {
	const gameIds = useAtomValue(gameIdsAtom);
	const counts: TabCounts = useMemo(
		() => ({
			installedGames: gameIds.length,
			ownedGames: gameIds.length,
			mods: -1,
			settings: -1,
			thanks: -1,
		}),
		[gameIds],
	);

	return counts;
}
