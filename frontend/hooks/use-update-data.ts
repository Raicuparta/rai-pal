import { useCallback, useEffect } from "react";
import { useSetAtom } from "jotai";
import { commands, Result, Error } from "@api/bindings";
import { loadingCountAtom } from "./use-data";
import { showAppNotification } from "@components/app-notifications";

export function useUpdateData(executeOnMount = false) {
	const setLoading = useSetAtom(loadingCountAtom);

	const updateAppData = useCallback(() => {
		console.log("Updating app data...");

		function updateDataPart(promise: Promise<Result<null, Error>>) {
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

		updateDataPart(commands.updateLocalMods());
		updateDataPart(commands.updateRemoteGames());
		updateDataPart(commands.getProviderGames("Steam"));
		updateDataPart(commands.getProviderGames("Epic"));
		updateDataPart(commands.getProviderGames("Itch"));
		updateDataPart(commands.getProviderGames("Xbox"));
		updateDataPart(commands.getProviderGames("Manual"));
		updateDataPart(commands.getProviderGames("Gog"));
	}, [setLoading]);

	useEffect(() => {
		if (executeOnMount) {
			updateAppData();
		}
	}, [executeOnMount, updateAppData]);

	return updateAppData;
}
