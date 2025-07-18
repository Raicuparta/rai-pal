import { commands, ProviderId } from "@api/bindings";
import { useAppEvent } from "./use-app-event";
import { useCommandData } from "./use-command-data";
import { useCallback } from "react";

export function useGame(providerId: ProviderId, gameId: string) {
	const getGame = useCallback(
		() => commands.getGame(providerId, gameId),
		[providerId, gameId],
	);

	const [game, updateGame] = useCommandData(getGame, null);

	useAppEvent(
		"refreshGame",
		`game-${providerId}:${gameId}`,
		([refreshedProviderId, refreshedGameId]) => {
			if (refreshedProviderId !== providerId || refreshedGameId !== gameId)
				return;
			updateGame();
		},
	);

	return game;
}
