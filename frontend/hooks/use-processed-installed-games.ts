import { useAtomValue } from "jotai";
import { useMemo } from "react";
import { installedGamesAtom, ownedGamesAtom } from "./use-data";
import { useUnifiedMods } from "./use-unified-mods";
import { getIsOutdated } from "../util/is-outdated";
import { InstalledGame, OwnedGame } from "@api/bindings";

type InstalledModVersions = Record<string, string | null>;
type ProcessedInstalledGameRecord = Record<string, ProcessedInstalledGame>;
export interface ProcessedInstalledGame extends InstalledGame {
	hasOutdatedMod: boolean;
	ownedGame?: OwnedGame;
}

export function useProcessedInstalledGames() {
	const installedGames = useAtomValue(installedGamesAtom);
	const ownedGames = useAtomValue(ownedGamesAtom);
	const mods = useUnifiedMods();
	const processedInstalledGames: ProcessedInstalledGameRecord = useMemo(() => {
		const result: ProcessedInstalledGameRecord = {};

		for (const [gameId, installedGame] of Object.entries(installedGames)) {
			const installedModVersions: InstalledModVersions = {};
			for (const mod of Object.values(mods)) {
				const modEngine = mod.common.engine;
				const gameEngine = installedGame.executable.engine?.brand;
				const engineMatch =
					!modEngine || !gameEngine || modEngine == gameEngine;

				const modBackend = mod.common.unityBackend;
				const gameBackend = installedGame.executable.scriptingBackend;
				const backendMatch =
					!modBackend || !gameBackend || modBackend == gameBackend;

				if (backendMatch && engineMatch) {
					installedModVersions[mod.common.id] =
						mod.local?.manifest?.version ?? null;
				}
			}

			result[gameId] = {
				...installedGame,
				ownedGame: installedGame.ownedGameId
					? ownedGames[installedGame.ownedGameId]
					: undefined,
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
	}, [installedGames, mods, ownedGames]);

	return processedInstalledGames;
}
