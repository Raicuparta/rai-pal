import { useAtomValue } from "jotai";
import { useMemo } from "react";
import { providerDataAtom } from "./use-data";
import { PageId } from "../pages";

export type TabCounts = Record<PageId, number>;

export function useDataCounts() {
	const providerDataMap = useAtomValue(providerDataAtom);
	const counts: TabCounts = useMemo(
		() => ({
			installedGames: Object.values(providerDataMap)
				.map((data) => Object.entries(data.installedGames))
				.flat().length,
			ownedGames: Object.values(providerDataMap)
				.map((data) => Object.entries(data.ownedGames))
				.flat().length,
			mods: -1,
			settings: -1,
			thanks: -1,
		}),
		[providerDataMap],
	);

	return counts;
}
