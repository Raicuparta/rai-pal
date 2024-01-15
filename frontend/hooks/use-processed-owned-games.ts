import { useAtomValue } from "jotai";
import { useMemo } from "react";
import { ownedGamesAtom, installedGamesAtom } from "./use-data";
import { InstalledGame, OwnedGame, ProviderId } from "@api/bindings";

type ProcessedOwnedGameRecord = Record<string, ProcessedOwnedGame>;
export interface ProcessedOwnedGame extends OwnedGame {
	isInstalled: boolean;
}

type InstalledGamesByProvider = Record<
	ProviderId,
	Record<string, InstalledGame>
>;

export function useProcessedOwnedGames() {
	const ownedGames = useAtomValue(ownedGamesAtom);
	const installedGames = useAtomValue(installedGamesAtom);

	const installedGamesByProvider: InstalledGamesByProvider = useMemo(() => {
		console.log("recalculating processed owned games");
		const result: InstalledGamesByProvider = {
			Steam: {},
			Manual: {},
			Epic: {},
			Gog: {},
			Xbox: {},
		};

		for (const installedGame of Object.values(installedGames)) {
			if (!installedGame.ownedGameId) continue;

			result[installedGame.provider][installedGame.ownedGameId] = installedGame;
		}

		return result;
	}, [installedGames]);

	const processedOwnedGames: ProcessedOwnedGameRecord = useMemo(() => {
		const result: ProcessedOwnedGameRecord = {};

		for (const [gameId, ownedGame] of Object.entries(ownedGames)) {
			const installedGame =
				installedGamesByProvider[ownedGame.provider][ownedGame.id];

			result[gameId] = {
				...ownedGame,
				isInstalled: Boolean(installedGame),
			};
		}

		return result;
	}, [ownedGames, installedGamesByProvider]);

	return processedOwnedGames;
}
