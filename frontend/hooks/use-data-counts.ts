import { useAtomValue } from "jotai";
import { useMemo } from "react";
import { providerDataAtom } from "./use-data";
import { PageId } from "../pages";

export type TabCounts = Record<PageId, number>;

export function useDataCounts() {
	const providerDataMap = useAtomValue(providerDataAtom);
	const counts: TabCounts = useMemo(
		() => ({
			installedGames: providerDataMap.installedGames.length,
			ownedGames: providerDataMap.ownedGames.length,
			mods: -1,
			settings: -1,
			thanks: -1,
		}),
		[providerDataMap],
	);

	return counts;
}
