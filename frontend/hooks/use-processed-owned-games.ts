import { useAtomValue } from "jotai";
import { useMemo } from "react";
import { remoteGamesAtom, providerDataAtom } from "./use-data";
import { OwnedGame, ProviderId, RemoteGame } from "@api/bindings";

type ProcessedOwnedGameRecord = Record<string, ProcessedOwnedGame>;
export interface ProcessedOwnedGame extends OwnedGame {
	isInstalled: boolean;
	remoteData?: RemoteGame;
}

export function useProcessedOwnedGames() {
	const providerDataMap = useAtomValue(providerDataAtom);
	const remoteGames = useAtomValue(remoteGamesAtom);

	const [databaseGamesByProvider, databaseGamesByTitle] = useMemo(() => {
		const byProvider: Partial<Record<ProviderId, Record<string, RemoteGame>>> =
			{};
		const byTitle: Record<string, RemoteGame> = {};
		const providerIds = Object.keys(providerDataMap) as ProviderId[];

		for (const remoteGame of remoteGames) {
			for (const providerId of providerIds) {
				byProvider[providerId] ??= {};
				for (const remoteGameId of remoteGame.providerIds[providerId] ?? []) {
					byProvider[providerId][remoteGameId] = remoteGame;
				}
			}

			if (remoteGame.title) {
				for (const normalizedTitle of remoteGame.title.normalized) {
					byTitle[normalizedTitle] = remoteGame;
				}
			}
		}
		
		return [byProvider, byTitle];
	}, [remoteGames, providerDataMap]);

	// Global IDs of owned games that are also installed.
	const installedOwnedIds: Set<string> = useMemo(() => {
		const result: Set<string> = new Set();

		for (const providerData of Object.values(providerDataMap)) {
			for (const installedGame of Object.values(providerData.installedGames)) {
				if (!installedGame.ownedGameId) continue;

				result.add(installedGame.ownedGameId);
			}
		}

		return result;
	}, [providerDataMap]);

	const processedOwnedGames: ProcessedOwnedGameRecord = useMemo(() => {
		const result: ProcessedOwnedGameRecord = {};

		function getRemoteDataByTitle(ownedGame: OwnedGame) {
			for (const normalizedTitle of ownedGame.title.normalized) {
				if (databaseGamesByTitle[normalizedTitle]) {
					// We use the first normalized title we have that returns a result.
					// That means the order returned from the backend matters (first has priority).
					return databaseGamesByTitle[normalizedTitle];
				}
			}
		}

		for (const providerData of Object.values(providerDataMap)) {
			for (const [gameId, ownedGame] of Object.entries(
				providerData.ownedGames,
			)) {
				const provider = providerDataMap[ownedGame.provider];
				if (!provider) continue;

				const remoteData =
					databaseGamesByProvider[ownedGame.provider]?.[
						ownedGame.providerGameId
					] ?? getRemoteDataByTitle(ownedGame);

				result[gameId] = {
					...ownedGame,
					remoteData,
					isInstalled: installedOwnedIds.has(ownedGame.globalId),
				};
			}
		}

		return result;
	}, [
		databaseGamesByProvider,
		databaseGamesByTitle,
		installedOwnedIds,
		providerDataMap,
	]);

	return processedOwnedGames;
}
