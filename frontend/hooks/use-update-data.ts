import {
	useCallback,
	useDeferredValue,
	useEffect,
	useRef,
	useState,
} from "react";
import { useAtomValue, useSetAtom } from "jotai";
import { commands, Result, Error, ProviderId } from "@api/bindings";
import { loadingCountAtom, gameDataAtom } from "./use-data";
import { showAppNotification } from "@components/app-notifications";
import { useAppEvent } from "./use-app-event";
import { gamesQueryAtom } from "./use-data-query";
import { useThrottledCallback } from "@mantine/hooks";

export function useUpdateData(executeOnMount = false) {
	const setLoading = useSetAtom(loadingCountAtom);
	const setGameData = useSetAtom(gameDataAtom);
	const gamesQuery = useAtomValue(gamesQueryAtom);
	const deferredGamesQuery = useDeferredValue(gamesQuery);
	const [providerIds, setProviderIds] = useState<ProviderId[]>([]);
	const fetchCount = useRef(0);

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
		fetchCount.current++;
		const thisFetchCount = fetchCount.current;
		commands.getGameIds(deferredGamesQuery).then((result) => {
			if (thisFetchCount !== fetchCount.current) {
				console.log(
					"Cancelling this fetch since another one happened in the meantime.",
				);
				return false;
			}

			if (result.status === "error") {
				showAppNotification(`Failed to get app data: ${result.error}`, "error");
				return false;
			}

			setGameData(result.data);

			return true;
		});
	}, [deferredGamesQuery, setGameData]);

	const throttledUpdateProviderGames = useThrottledCallback(
		updateProviderGames,
		1000,
	);

	useEffect(() => {
		updateProviderGames();
	}, [updateProviderGames]);

	useAppEvent("gamesChanged", throttledUpdateProviderGames);

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
			handleDataPromise(commands.refreshGames(providerId));
		}

		handleDataPromise(commands.refreshMods());
		handleDataPromise(commands.refreshRemoteGames());
	}, [providerIds, setLoading]);

	useEffect(() => {
		if (executeOnMount) {
			updateAppData();
		}
	}, [executeOnMount, updateAppData]);

	return updateAppData;
}
