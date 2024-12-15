import { useCallback, useEffect, useState } from "react";
import { useSetAtom } from "jotai";
import { commands, Result, Error, events, ProviderId } from "@api/bindings";
import { loadingCountAtom, gameIdsAtom } from "./use-data";
import { showAppNotification } from "@components/app-notifications";
import { useAppEvent } from "./use-app-event";

export function useUpdateData(executeOnMount = false) {
	const setLoading = useSetAtom(loadingCountAtom);
	const setGameIds = useSetAtom(gameIdsAtom);
	const [providerIds, setProviderIds] = useState<ProviderId[]>([]);

	useEffect(() => {
		commands.getProviderIds().then((providerIdsResult) => {
			if (providerIdsResult.status === "error") {
				showAppNotification(
					`Failed to get info about available game providers: ${providerIdsResult.error}`,
					"error",
				);
				return;
			}

			setProviderIds(providerIdsResult.data);
		});
	}, []);

	const updateProviderGames = useCallback(() => {
		commands.getData().then((result) => {
			if (result.status === "error") {
				showAppNotification(`Failed to get app data: ${result.error}`, "error");
				return false;
			}

			setGameIds(result.data);

			return true;
		});
	}, [setGameIds]);

	useAppEvent(events.foundOwnedGame, updateProviderGames);
	useAppEvent(events.foundInstalledGame, updateProviderGames);
	useAppEvent(events.foundOwnedGame, updateProviderGames);

	const updateAppData = useCallback(() => {
		function handleDataPromise(promise: Promise<Result<null, Error>>) {
			setLoading((previousLoading) => previousLoading + 1);
			promise
				.then((result) => {
					if (result.status === "error") {
						showAppNotification(
							`Error while updating data: ${result.error}`,
							"error",
						);
					}
				})
				.catch((error) => {
					showAppNotification(
						`Failed to initialize data update: ${error}`,
						"error",
					);
				})
				.finally(() => setLoading((previousLoading) => previousLoading - 1));
		}

		for (const providerId of providerIds) {
			handleDataPromise(commands.getProviderGames(providerId));
		}

		handleDataPromise(commands.updateLocalMods());
	}, [providerIds, setLoading]);

	useEffect(() => {
		if (executeOnMount) {
			updateAppData();
		}
	}, [executeOnMount, updateAppData]);

	return updateAppData;
}
