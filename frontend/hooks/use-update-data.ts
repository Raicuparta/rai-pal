import { useCallback, useEffect } from "react";
import { useSetAtom } from "jotai";
import { commands, Result, Error } from "@api/bindings";
import { loadingCountAtom } from "./use-data";
import { showAppNotification } from "@components/app-notifications";

export function useUpdateData(executeOnMount = false) {
	const setLoading = useSetAtom(loadingCountAtom);

	const updateAppData = useCallback(() => {
		console.log("Updating app data...");

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

		commands.getProviderIds().then((providerIdsResult) => {
			if (providerIdsResult.status === "error") {
				showAppNotification(
					`Failed to get info about available game providers: ${providerIdsResult.error}`,
					"error",
				);
				return;
			}

			for (const providerId of providerIdsResult.data) {
				handleDataPromise(commands.getProviderGames(providerId));
			}

			handleDataPromise(commands.updateLocalMods());
			// handleDataPromise(commands.updateRemoteGames());
		});
	}, [setLoading]);

	useEffect(() => {
		if (executeOnMount) {
			updateAppData();
		}
	}, [executeOnMount, updateAppData]);

	return updateAppData;
}
