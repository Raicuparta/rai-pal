import { commands, Game, ProviderId } from "@api/bindings";
import { useEffect, useMemo, useState } from "react";

export function useGame(providerId: ProviderId, index: bigint) {
	const defaultGame: Game = useMemo(
		() => ({
			id: "",
			providerId,
			installedGame: null,
			ownedGame: null,
			fromSubscriptions: [],
			providerCommands: {},
			releaseDate: null,
			tags: [],
			thumbnailUrl: null,
			title: {
				display: "...",
				normalized: ["..."],
			},
		}),
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
