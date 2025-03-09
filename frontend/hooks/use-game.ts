import { commands, Game, GameId } from "@api/bindings";
import { useCallback, useEffect, useState } from "react";
import { useAsyncCommand } from "./use-async-command";
import { useAppEvent } from "./use-app-event";

export function useGame({ providerId, gameId }: GameId) {
	const [getGame] = useAsyncCommand(commands.getGame);
	const defaultGame: Game = {
		id: { gameId, providerId },
		externalId: gameId,
		installedGame: null,
		remoteGame: null,
		fromSubscriptions: [],
		releaseDate: null,
		tags: [],
		thumbnailUrl: null,
		title: {
			display: "...",
			normalized: ["..."],
		},
		providerCommands: {},
	};

	const [game, setGame] = useState<Game>(defaultGame);

	const updateData = useCallback(() => {
		getGame({ providerId, gameId }).then(setGame);
	}, [gameId, getGame, providerId]);

	useEffect(() => {
		updateData();
	}, [updateData]);

	useAppEvent("foundGame", `${providerId}:${gameId}`, (foundId) => {
		if (foundId.providerId !== providerId || foundId.gameId !== gameId) return;
		updateData();
	});

	return game;
}
