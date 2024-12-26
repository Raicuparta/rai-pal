import { commands, Game, ProviderId } from "@api/bindings";
import { useEffect, useMemo, useState } from "react";

export function useGame(providerId: ProviderId, gameId: string) {
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

	useEffect(() => {
		console.log(`fetching game ${gameId}`);
		commands.getGame(providerId, gameId).then((result) => {
			console.log("it ended", result);
			if (result.status === "ok" && result.data) {
				if (`${gameId}` === "1452") {
					console.log(`fetched game ${result.data}`);
				}
				setGame(result.data);
			} else if (result.status === "error") {
				console.error(`Failed to fetch game: ${result.error}`);
			}
		});
	}, [gameId, providerId]);

	// useAppEvent(events.foundGame, updateData); // TODO

	return game;
}
