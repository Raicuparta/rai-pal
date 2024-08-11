import { useAtomValue } from "jotai";
import { useMemo } from "react";
import { installedGamesAtom, ownedGamesAtom } from "./use-data";
import { PageId } from "../pages";

export type TabCounts = Record<PageId, number>;

export function useDataCounts() {
	const installedGames = useAtomValue(installedGamesAtom);
	const ownedGames = useAtomValue(ownedGamesAtom);
	const counts: TabCounts = useMemo(
		() => ({
			installedGames: Object.entries(installedGames).length,
			ownedGames: Object.entries(ownedGames).length,
			mods: -1,
			settings: -1,
			thanks: -1,
		}),
		[installedGames, ownedGames],
	);

	return counts;
}
