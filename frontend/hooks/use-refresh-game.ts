import { commands, GameProviderId } from "@api/bindings";
import { showAppNotification } from "@components/app-notifications";
import { useCallback } from "react";
import { useRefreshTask } from "./use-refresh-tasks";

export function useRefreshGame(providerId: GameProviderId, gameId: string) {
	const runRefreshTask = useRefreshTask();

	return useCallback(async () => {
		try {
			await commands.refreshGame(providerId, gameId);
		} catch (error) {
			showAppNotification(`Failed to refresh game: ${error}`, "error");
			return;
		}

		return runRefreshTask("mods", commands.refreshMods());
	}, [providerId, gameId, runRefreshTask]);
}
