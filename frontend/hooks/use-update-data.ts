import { useCallback, useDeferredValue, useEffect, useRef } from "react";
import { useSetAtom } from "jotai";
import { commands } from "@api/bindings";
import { loadingTasksAtom, gameDataAtom } from "./use-data";
import { showAppNotification } from "@components/app-notifications";
import { useAppEvent } from "./use-app-event";
import { useThrottledCallback } from "@mantine/hooks";
import { useDataQuery } from "./use-data-query";

export function useUpdateData(executeOnMount = false) {
	const setLoading = useSetAtom(loadingTasksAtom);
	const setGameData = useSetAtom(gameDataAtom);
	const [gamesQuery] = useDataQuery();
	const deferredGamesQuery = useDeferredValue(gamesQuery);
	const fetchCount = useRef(0);
	const loadingTaskCount = useRef(0);
	const hasExecutedOnMount = useRef(false);

	const updateProviderGames = useCallback(() => {
		fetchCount.current++;
		const thisFetchCount = fetchCount.current;
		commands
			.getGameIds(deferredGamesQuery)
			.then((data) => {
				if (thisFetchCount !== fetchCount.current) {
					console.log(
						"Cancelling this fetch since another one happened in the meantime.",
					);
					return;
				}

				setGameData(data);
			})
			.catch((error) => {
				showAppNotification(`Failed to get app data: ${error}`, "error");
			});
	}, [deferredGamesQuery, setGameData]);

	const throttledUpdateProviderGames = useThrottledCallback(
		updateProviderGames,
		1000,
	);

	useEffect(() => {
		updateProviderGames();
	}, [updateProviderGames]);

	useAppEvent("gamesChanged", "update-data", throttledUpdateProviderGames);

	const updateAppData = useCallback(async () => {
		const providerIds = await commands.getProviderIds().catch((error) => {
			showAppNotification(
				`Failed to get info about available game providers: ${error}`,
				"error",
			);
			return null;
		});

		if (!providerIds) {
			console.error("No providers available, skipping data update.");
			return;
		}

		function handleDataPromise(promise: Promise<null>, taskName: string) {
			loadingTaskCount.current += 1;
			const taskIndex = loadingTaskCount.current;
			setLoading((previousLoadingTasks) => [
				...previousLoadingTasks,
				{ name: taskName, index: taskIndex },
			]);
			promise
				.catch((error) => {
					showAppNotification(
						`Failed to initialize data update: ${error}`,
						"error",
					);
				})
				.finally(() =>
					setLoading((previousLoadingTasks) =>
						previousLoadingTasks.filter((task) => task.index !== taskIndex),
					),
				);
		}

		for (const providerId of providerIds) {
			handleDataPromise(commands.refreshGames(providerId), providerId);
		}

		handleDataPromise(commands.refreshMods(), "mods");
		handleDataPromise(commands.refreshRemoteGames(), "remote data");
	}, [setLoading]);

	useEffect(() => {
		if (executeOnMount && !hasExecutedOnMount.current) {
			updateAppData();
			hasExecutedOnMount.current = true;
		}
	}, [executeOnMount, updateAppData]);

	return updateAppData;
}
