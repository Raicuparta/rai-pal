import { commands, events, Game, ProviderId } from "@api/bindings";
import { useCallback, useEffect, useState } from "react";
import { useAppEvent } from "./use-app-event";

export function useGame(provider: ProviderId, index: bigint) {
	const [game, setGame] = useState<Game>();

	const updateData = useCallback(() => {
		commands.getGame(provider, index).then((result) => {
			if (result.status === "ok" && result.data) {
				setGame(result.data);
			}
		});
	}, [index, provider]);

	useEffect(updateData, [updateData]);

	useAppEvent(events.foundInstalledGame, updateData);

	return game;
}