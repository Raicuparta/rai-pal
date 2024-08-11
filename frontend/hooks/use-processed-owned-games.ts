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

type DatabaseMapGroupBy = "steamId" | "epicId" | "gogId" | "title";

function normalizeTitle(title: string): string {
	return title.replace(/\W+/g, "").toLowerCase();
}

export function useProcessedOwnedGames() {
	const ownedGames = useAtomValue(ownedGamesAtom);
	const installedGames = useAtomValue(installedGamesAtom);
	const remoteGames = useAtomValue(remoteGamesAtom);

	const databaseGamesByProvider = useMemo(() => {
		const result: Record<DatabaseMapGroupBy, Record<string, RemoteGame>> = {
			steamId: {},
			epicId: {},
			gogId: {},
			title: {},
		};

		for (const remoteGame of remoteGames) {
			for (const steamId of remoteGame.steamIds ?? []) {
				result.steamId[steamId] = remoteGame;
			}
			for (const gogId of remoteGame.gogIds ?? []) {
				result.gogId[gogId] = remoteGame;
			}
			for (const epicId of remoteGame.epicIds ?? []) {
				result.epicId[epicId] = remoteGame;
			}

			if (remoteGame.title) {
				result.title[normalizeTitle(remoteGame.title)] = remoteGame;
			}
		}

		return result;
	}, [remoteGames]);

	const installedGamesByProvider: InstalledGamesByProvider = useMemo(() => {
		const result: InstalledGamesByProvider = {
			Steam: {},
			Manual: {},
			Epic: {},
			Gog: {},
			Xbox: {},
			Itch: {},
		};

		for (const installedGame of Object.values(installedGames)) {
			if (!installedGame.ownedGameId) continue;

			result[installedGame.provider][installedGame.ownedGameId] = installedGame;
		}

		return result;
	}, [installedGames]);

	const processedOwnedGames: ProcessedOwnedGameRecord = useMemo(() => {
		const result: ProcessedOwnedGameRecord = {};

		function getDatabaseGameMapping(ownedGame: OwnedGame) {
			switch (ownedGame.provider) {
				case "Steam":
					return databaseGamesByProvider.steamId;
				case "Epic":
					return databaseGamesByProvider.epicId;
				case "Gog":
					return databaseGamesByProvider.gogId;
				default:
					return databaseGamesByProvider.title;
			}
		}

		function getDatabaseGame(ownedGame: OwnedGame) {
			const map = getDatabaseGameMapping(ownedGame);
			const key =
				map === databaseGamesByProvider.title
					? normalizeTitle(ownedGame.name)
					: ownedGame.providerGameId;

			return map[key];
		}

		for (const [gameId, ownedGame] of Object.entries(ownedGames)) {
			const installedGame =
				installedGamesByProvider[ownedGame.provider][ownedGame.globalId];

			result[gameId] = {
				...ownedGame,
				remoteData: getDatabaseGame(ownedGame),
				isInstalled: Boolean(installedGame),
			};
		}

		return result;
	}, [ownedGames, installedGamesByProvider, databaseGamesByProvider]);

	return processedOwnedGames;
}
