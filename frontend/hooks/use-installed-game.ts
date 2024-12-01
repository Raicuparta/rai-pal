import { commands, events, InstalledGame, ProviderId } from "@api/bindings";
import { useCallback, useEffect, useState } from "react";
import { useAppEvent } from "./use-app-event";

export function useInstalledGame(provider: ProviderId, gameId: string) {
	const [game, setGame] = useState<InstalledGame>();

	const updateData = useCallback(() => {
		commands.getInstalledGame(provider, gameId).then((result) => {
			if (result.status === "ok") {
				setGame(result.data);
			}
		});
	}, [provider, gameId]);

	useEffect(updateData, [updateData]);

	useAppEvent(events.foundInstalledGame, updateData);

	return game;
}
