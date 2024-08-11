import { useCallback, useEffect, useState } from "react";
import { useSetAtom } from "jotai";
import { commands, Result, Error, events, ProviderId } from "@api/bindings";
import {
	installedGamesAtom,
	loadingCountAtom,
	ownedGamesAtom,
} from "./use-data";
import { showAppNotification } from "@components/app-notifications";
import { useAppEvent } from "./use-app-event";
import { useThrottledCallback } from "@mantine/hooks";

export function useUpdateData(executeOnMount = false) {
	const setLoading = useSetAtom(loadingCountAtom);
	const setInstalledGames = useSetAtom(installedGamesAtom);
	const setOwnedGames = useSetAtom(ownedGamesAtom);
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
		for (const providerId of providerIds) {
			commands.getProviderCache(providerId).then((cacheResult) => {
				if (cacheResult.status === "error") {
					showAppNotification(
						`Failed to get provider cache for ${providerId}: ${cacheResult.error}`,
						"error",
					);
					return false;
				}

				setInstalledGames((previousInstalledGames) => ({
					...previousInstalledGames,
					...cacheResult.data.installedGames,
				}));

				setOwnedGames((previousOwnedGames) => ({
					...previousOwnedGames,
					...cacheResult.data.ownedGames,
				}));

				return true;
			});
		}
	}, [providerIds, setInstalledGames, setOwnedGames]);

	const throttledUpdateProviderGames = useThrottledCallback(
		updateProviderGames,
		1000,
	);

	useAppEvent(events.foundOwnedGame, throttledUpdateProviderGames);
	useAppEvent(events.foundInstalledGame, throttledUpdateProviderGames);

	useEffect(() => {
		updateProviderGames();
	}, [updateProviderGames]);

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
