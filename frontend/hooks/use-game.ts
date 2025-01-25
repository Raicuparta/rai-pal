import { commands, Game, GameId } from "@api/bindings";
import { useCallback, useEffect, useMemo, useState } from "react";
import { useAsyncCommand } from "./use-async-command";
import { useAppEvent } from "./use-app-event";

export function useGame({ providerId, gameId }: GameId) {
	const [getGame] = useAsyncCommand(commands.getGame);
	const defaultGame: Game = useMemo(
		() => ({
			id: { gameId, providerId },
			externalId: gameId,
			installedGame: null,
			ownedGame: null,
			remoteGame: null,
			fromSubscriptions: [],
			releaseDate: null,
			tags: [],
			thumbnailUrl: null,
			title: {
				display: "...",
				normalized: ["..."],
			},

			// TODO this "as" won't be needed after specta makes maps Partial.
			providerCommands: {} as Game["providerCommands"],
		}),
		[gameId, providerId],
	);

	const [game, setGame] = useState<Game>(defaultGame);

	const updateData = useCallback(() => {
		getGame({ providerId, gameId }).then(setGame);
	}, [getGame, providerId, gameId]);

	useEffect(() => {
		updateData();
	}, [updateData]);

	const foundGameCallback = useCallback(
		(foundId: GameId) => {
			if (foundId.providerId !== providerId || foundId.gameId !== gameId)
				return;
			updateData();
		},
		[gameId, providerId, updateData],
	);

	useAppEvent("foundGame", `${providerId}:${gameId}`, foundGameCallback);

	return game;
}
