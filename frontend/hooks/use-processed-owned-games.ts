import { useAtomValue } from "jotai";
import { useMemo } from "react";
import {
	ownedGamesAtom,
	installedGamesAtom,
	gameDatabaseAtom,
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
	const gameDatabase = useAtomValue(gameDatabaseAtom);

	const databaseGamesByProvider = useMemo(() => {
		const result: Record<DatabaseMapGroupBy, Record<string, RemoteGame>> = {
			steamId: {},
			epicId: {},
			gogId: {},
			title: {},
		};

		for (const gameDatabaseEntry of gameDatabase) {
			for (const steamId of gameDatabaseEntry.steamIds ?? []) {
				result.steamId[steamId] = gameDatabaseEntry;
			}
			for (const gogId of gameDatabaseEntry.gogIds ?? []) {
				result.gogId[gogId] = gameDatabaseEntry;
			}
			for (const epicId of gameDatabaseEntry.epicIds ?? []) {
				result.epicId[epicId] = gameDatabaseEntry;
			}

			if (gameDatabaseEntry.title) {
				result.title[normalizeTitle(gameDatabaseEntry.title)] =
					gameDatabaseEntry;
			}
		}

		return result;
	}, [gameDatabase]);

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
					: ownedGame.id.replace(`${ownedGame.provider}_`, "");

			if (map === databaseGamesByProvider.title) {
				console.log(
					"map === databaseGamesByProvider.title",
					key,
					"and...",
					map[key],
				);
			}

			return map[key];
		}

		for (const [gameId, ownedGame] of ownedGames.data.entries()) {
			const installedGame =
				installedGamesByProvider[ownedGame.provider][ownedGame.id];

			result[gameId] = {
				...ownedGame,
				remoteData: getDatabaseGame(ownedGame),
				isInstalled: Boolean(installedGame),
			};
		}

		return result;
	}, [ownedGames.data, installedGamesByProvider, databaseGamesByProvider]);

	return processedOwnedGames;
}
