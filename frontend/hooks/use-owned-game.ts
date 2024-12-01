import { commands, events, OwnedGame, ProviderId } from "@api/bindings";
import { useCallback, useEffect, useState } from "react";
import { useAppEvent } from "./use-app-event";

export function useOwnedGame(provider: ProviderId, gameId: string) {
	const [game, setGame] = useState<OwnedGame>();

	const updateData = useCallback(() => {
		commands.getOwnedGame(provider, gameId).then((result) => {
			if (result.status === "ok") {
				setGame(result.data);
			}
		});
	}, [provider, gameId]);

	useEffect(updateData, [updateData]);

	useAppEvent(events.foundOwnedGame, updateData);

	return game;
}
