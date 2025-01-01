import { commands, Game, ProviderId } from "@api/bindings";
import { useCallback, useEffect, useMemo, useState } from "react";
import { useAsyncCommand } from "./use-async-command";

export function useGame(providerId: ProviderId, gameId: string) {
	const [getGame] = useAsyncCommand(commands.getGame);
	const defaultGame: Game = useMemo(
		() =>
			({
				id: "",
				providerId,
				installedGame: null,
				ownedGame: null,
				remoteGame: null,
				fromSubscriptions: [],
				providerCommands: {},
				releaseDate: null,
				tags: [],
				thumbnailUrl: null,
				title: {
					display: "...",
					normalized: ["..."],
				},
			}) as unknown as Game, // TODO this won't be needed after specta makes maps Partial.
		[providerId],
	);

	const [game, setGame] = useState<Game>(defaultGame);

	const updateData = useCallback(() => {
		getGame({ providerId, gameId }).then((game) => {
			setGame(game);
		});
	}, [gameId, getGame, providerId]);

	useEffect(() => {
		updateData();
	}, [updateData]);

	// TODO: this makes the app super slow due to subscribing to events too often.
	// We need to make a single top level frontend event that handles all this.
	// useAppEvent(events.foundGame, updateData);

	return game;
}
