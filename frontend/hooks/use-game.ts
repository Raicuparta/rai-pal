import { commands, Game, ProviderId } from "@api/bindings";
import { useEffect, useMemo, useState } from "react";

export function useGame(providerId: ProviderId, index: bigint) {
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
		commands.getGame(providerId, index).then((result) => {
			if (result.status === "ok" && result.data) {
				setGame(result.data);
			}
		});
	}, [index, providerId]);

	// useAppEvent(events.foundGame, updateData); // TODO

	return game;
}
