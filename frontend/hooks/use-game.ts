import { commands, DbGame, GameProviderId } from "@api/bindings";
import { useAppEvent } from "./use-app-event";
import { useCommandData } from "./use-command-data";
import { useAtomValue } from "jotai";
import { gameDataVersionAtom } from "./use-data";
import { useCallback, useEffect, useMemo, useRef } from "react";

let nextGameHookId = 0;

const gameCache = new Map<string, DbGame>();

export function useGame(providerId?: GameProviderId, gameId?: string) {
	const hookId = useMemo(() => nextGameHookId++, []);
	const cacheKey = `${providerId}:${gameId}`;

	const getGame = useCallback(async () => {
		if (!providerId || !gameId) return null;
		try {
			return await commands.getGame(providerId, gameId);
		} catch {
			return null;
		}
	}, [providerId, gameId]);

	const [game, updateGame] = useCommandData(
		getGame,
		gameCache.get(cacheKey) ?? null,
	);

	useEffect(() => {
		if (game) {
			gameCache.set(cacheKey, game);
		}
	}, [game, cacheKey]);

	useAppEvent(
		"refreshGame",
		`game-${providerId}:${gameId}:${hookId}`,
		([refreshedProviderId, refreshedGameId]) => {
			if (refreshedProviderId !== providerId || refreshedGameId !== gameId)
				return;
			updateGame();
		},
	);

	const version = useAtomValue(gameDataVersionAtom);
	const lastFetchedVersion = useRef(version);
	useEffect(() => {
		if (lastFetchedVersion.current >= version) return;
		lastFetchedVersion.current = version;
		updateGame();
	}, [version, updateGame]);

	return game;
}
