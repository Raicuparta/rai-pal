import { useAtomValue } from "jotai";
import { useMemo } from "react";
import { installedGamesAtom } from "./use-data";
import { useUnifiedMods } from "./use-unified-mods";
import { getIsOutdated } from "../util/is-outdated";
import { InstalledGame } from "@api/bindings";

type ProcessedInstalledGameRecord = Record<string, ProcessedInstalledGame>;
export interface ProcessedInstalledGame extends InstalledGame {
	hasOutdatedMod: boolean;
}

export function useProcessedInstalledGames() {
	const installedGames = useAtomValue(installedGamesAtom);
	const mods = useUnifiedMods();
	const processedInstalledGames: ProcessedInstalledGameRecord = useMemo(() => {
		const result: ProcessedInstalledGameRecord = {};

		for (const [gameId, installedGame] of Object.entries(installedGames)) {
			result[gameId] = {
				...installedGame,
				hasOutdatedMod:
					Object.entries(installedGame.installedModVersions).findIndex(
						([modId, installedVersion]) =>
							getIsOutdated(
								installedVersion,
								mods[modId]?.remote?.latestVersion?.id,
							),
					) !== -1,
			};
		}

		return result;
	}, [installedGames, mods]);

	return processedInstalledGames;
}
