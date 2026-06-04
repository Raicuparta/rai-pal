import { useCallback, useEffect, useRef } from "react";
import { useSetAtom } from "jotai";
import { commands, PROVIDER_IDS } from "@api/bindings";
import { loadingTasksAtom, gameDataAtom, modsAtom } from "./use-data";
import { showAppNotification } from "@components/app-notifications";
import { useAppEvent } from "./use-app-event";
import { useThrottledCallback } from "@mantine/hooks";
import { useDataQuery } from "./use-data-query";

// TODO this is kinda stupid, hook doing too much work since it's used in multiple places.
export function useUpdateData(executeOnMount = false) {
	const setLoadingTasks = useSetAtom(loadingTasksAtom);
	const setGameData = useSetAtom(gameDataAtom);
	const setMods = useSetAtom(modsAtom);
	const [gamesQuery] = useDataQuery();
	const totalGameFetchCount = useRef(0);
	const totalModFetchCount = useRef(0);
	const totalLoadingTaskCount = useRef(0);
	const hasExecutedOnMount = useRef(false);

	const updateGames = useCallback(() => {
		totalGameFetchCount.current++;
		const thisFetchCount = totalGameFetchCount.current;
		commands
			.getGameIds(gamesQuery)
			.then((data) => {
				if (thisFetchCount !== totalGameFetchCount.current) {
					console.log(
						"Cancelling this fetch since another one happened in the meantime.",
					);
					return;
				}

				setGameData(data);
			})
			.catch((error) => {
				showAppNotification(`Failed to get app games data: ${error}`, "error");
			});
	}, [gamesQuery, setGameData]);

	const updateMods = useCallback(() => {
		totalModFetchCount.current++;
		const thisFetchCount = totalModFetchCount.current;
		commands
			.getMods()
			.then((data) => {
				if (thisFetchCount !== totalModFetchCount.current) {
					console.log(
						"Cancelling this fetch since another one happened in the meantime.",
					);
					return;
				}

				setMods(data);
			})
			.catch((error) => {
				showAppNotification(`Failed to get app mods data: ${error}`, "error");
			});
	}, [setMods]);

	const throttledUpdateData = useThrottledCallback(() => {
		updateGames();
		updateMods();
	}, 1000);

	useEffect(() => {
		if (!executeOnMount) return;
		updateGames();
		updateMods();
	}, [updateGames, executeOnMount, updateMods]);

	useAppEvent("appDatabaseChanged", "update-data", throttledUpdateData);

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
