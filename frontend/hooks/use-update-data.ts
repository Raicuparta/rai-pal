import { useCallback, useDeferredValue, useEffect, useRef } from "react";
import { useAtom, useSetAtom } from "jotai";
import { commands, PROVIDER_IDS } from "@api/bindings";
import { loadingTasksAtom, gameDataAtom } from "./use-data";
import { showAppNotification } from "@components/app-notifications";
import { useAppEvent } from "./use-app-event";
import { useThrottledCallback } from "@mantine/hooks";
import { useDataQuery } from "./use-data-query";

// TODO this is kinda stupid, hook doing too much work since it's used in multiple places.
export function useUpdateData(executeOnMount = false) {
	const [loadingTasks, setLoadingTasks] = useAtom(loadingTasksAtom);
	const setGameData = useSetAtom(gameDataAtom);
	const [gamesQuery] = useDataQuery();
	const deferredGamesQuery = useDeferredValue(gamesQuery);
	const totalFetchCount = useRef(0);
	const totalLoadingTaskCount = useRef(0);
	const hasExecutedOnMount = useRef(false);
	const isLoading = useRef(false);
	const refreshWhenLoadingDone = useRef(false);

	const updateProviderGames = useCallback(() => {
		totalFetchCount.current++;
		const thisFetchCount = totalFetchCount.current;
		commands
			.getGameIds(deferredGamesQuery)
			.then((data) => {
				console.log("gooot game ids!!!");
				if (thisFetchCount !== totalFetchCount.current) {
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
		if (!executeOnMount) return;
		updateProviderGames();
	}, [updateProviderGames, executeOnMount]);

	useAppEvent("gamesChanged", "update-data", throttledUpdateProviderGames);

	const updateAppData = useCallback(
		async (refreshDatabases: boolean) => {
			if (isLoading.current) {
				console.log(
					"Loading tasks are in progress, will refresh once loading is done.",
				);
				refreshWhenLoadingDone.current = true;
				return;
			}

			function handleDataPromise(promise: Promise<null>, taskName: string) {
				totalLoadingTaskCount.current += 1;
				const taskIndex = totalLoadingTaskCount.current;
				setLoadingTasks((previousLoadingTasks) => [
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
						setLoadingTasks((previousLoadingTasks) =>
							previousLoadingTasks.filter((task) => task.index !== taskIndex),
						),
					);
			}

			for (const providerId of PROVIDER_IDS) {
				handleDataPromise(commands.refreshGames(providerId), providerId);
			}

			if (refreshDatabases) {
				handleDataPromise(commands.refreshMods(), "mods");
				handleDataPromise(commands.refreshRemoteGames(), "remote data");
			}
		},
		[setLoadingTasks],
	);

	useEffect(() => {
		if (executeOnMount && !hasExecutedOnMount.current) {
			updateAppData(true);
			hasExecutedOnMount.current = true;
		}
	}, [executeOnMount, updateAppData]);

	useEffect(() => {
		isLoading.current = loadingTasks.length > 0;
		if (refreshWhenLoadingDone.current) {
			refreshWhenLoadingDone.current = false;
			updateAppData(false);
		}
	}, [loadingTasks.length, updateAppData]);

	return updateAppData;
}
