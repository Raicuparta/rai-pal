import { commands, DbGame, GameId } from "@api/bindings";
import { useCallback, useEffect, useState } from "react";
import { useAsyncCommand } from "./use-async-command";
import { useAppEvent } from "./use-app-event";

export function useGame({ providerId, gameId }: GameId) {
	const [getGame] = useAsyncCommand(commands.getGame);
	const defaultGame: DbGame = {
		providerId: providerId,
		gameId: gameId,
		displayTitle: "...",
		engineBrand: null,
		engineVersion: null,
		exePath: null,
		externalId: "",
		normalizedTitles: "",
		releaseDate: null,
		thumbnailUrl: null,
		architecture: null,
		unityBackend: null,
		tags: [],
	};

	const [game, setGame] = useState<DbGame>(defaultGame);

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
