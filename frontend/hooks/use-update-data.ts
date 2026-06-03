import { useCallback, useEffect, useRef } from "react";
import { useSetAtom } from "jotai";
import { commands, PROVIDER_IDS } from "@api/bindings";
import { loadingTasksAtom, gameDataAtom } from "./use-data";
import { showAppNotification } from "@components/app-notifications";
import { useAppEvent } from "./use-app-event";
import { useThrottledCallback } from "@mantine/hooks";
import { useDataQuery } from "./use-data-query";

// TODO this is kinda stupid, hook doing too much work since it's used in multiple places.
export function useUpdateData(executeOnMount = false) {
	const setLoadingTasks = useSetAtom(loadingTasksAtom);
	const setGameData = useSetAtom(gameDataAtom);
	const [gamesQuery] = useDataQuery();
	const totalFetchCount = useRef(0);
	const totalLoadingTaskCount = useRef(0);
	const hasExecutedOnMount = useRef(false);

	const updateGames = useCallback(() => {
		totalFetchCount.current++;
		const thisFetchCount = totalFetchCount.current;
		commands
			.getGameIds(gamesQuery)
			.then((data) => {
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
	}, [gamesQuery, setGameData]);

	const throttledUpdateProviderGames = useThrottledCallback(updateGames, 1000);

	useEffect(() => {
		if (!executeOnMount) return;
		updateGames();
	}, [updateGames, executeOnMount]);

	useAppEvent(
		"appDatabaseChanged",
		"update-data",
		throttledUpdateProviderGames,
	);

	const updateAppData = useCallback(async () => {
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
						`Failed to initialize data update (${taskName}): ${error}`,
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

		handleDataPromise(commands.refreshMods(), "mods");
		handleDataPromise(commands.refreshRemoteGames(), "remote data");
	}, [setLoadingTasks]);

	useEffect(() => {
		if (executeOnMount && !hasExecutedOnMount.current) {
			updateAppData();
			hasExecutedOnMount.current = true;
		}
	}, [executeOnMount, updateAppData]);

	return updateAppData;
}
