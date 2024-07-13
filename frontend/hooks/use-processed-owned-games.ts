import { useAtomValue } from "jotai";
import { useMemo } from "react";
import {
	ownedGamesAtom,
	installedGamesAtom,
	remoteGamesAtom,
} from "./use-data";
import {
	InstalledGame,
	OwnedGame,
	ProviderId,
	RemoteGame,
} from "@api/bindings";

type ProcessedOwnedGameRecord = Record<string, ProcessedOwnedGame>;
export interface ProcessedOwnedGame extends OwnedGame {
	isInstalled: boolean;
	remoteData?: RemoteGame;
}

type InstalledGamesByProvider = Record<
	ProviderId,
	Record<string, InstalledGame>
>;

export function useProcessedOwnedGames() {
	const ownedGames = useAtomValue(ownedGamesAtom);
	const remoteGames = useAtomValue(remoteGamesAtom);
	const installedGames = useAtomValue(installedGamesAtom);

	const installedGamesByProvider: InstalledGamesByProvider = useMemo(() => {
		const result: InstalledGamesByProvider = {
			Steam: {},
			Manual: {},
			Epic: {},
			Gog: {},
			Xbox: {},
			Itch: {},
		};

		for (const installedGame of installedGames.data.values()) {
			if (!installedGame.ownedGameId) continue;

			result[installedGame.provider][installedGame.ownedGameId] = installedGame;
		}

		return result;
	}, [installedGames]);

	const processedOwnedGames: ProcessedOwnedGameRecord = useMemo(() => {
		const result: ProcessedOwnedGameRecord = {};

		for (const [gameId, ownedGame] of ownedGames.data.entries()) {
			const installedGame =
				installedGamesByProvider[ownedGame.provider][ownedGame.id];

			result[gameId] = {
				...ownedGame,
				isInstalled: Boolean(installedGame),
				remoteData: remoteGames.data.get(ownedGame.id),
			};
		}

		return result;
	}, [ownedGames, installedGamesByProvider, remoteGames]);

	return processedOwnedGames;
}
