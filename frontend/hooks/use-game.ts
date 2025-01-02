import { commands, Game, GameId } from "@api/bindings";
import { useCallback, useEffect, useMemo, useState } from "react";
import { useAsyncCommand } from "./use-async-command";
import { useAppEvent } from "./use-app-event";

export function useGame(gameId: GameId) {
	const [getGame] = useAsyncCommand(commands.getGame);
	const defaultGame: Game = useMemo(
		() => ({
			id: gameId,
			externalId: gameId.gameId,
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
		[gameId],
	);

	const [game, setGame] = useState<Game>(defaultGame);

	const updateData = useCallback(() => {
		getGame(gameId).then((game) => {
			setGame(game);
		});
	}, [getGame, gameId]);

	useEffect(() => {
		updateData();
	}, [updateData]);

	useAppEvent(
		"foundGame",
		useCallback(
			(foundGameId) => {
				if (foundGameId !== gameId) return;
				updateData();
			},
			[gameId, updateData],
		),
	);

	return game;
}
