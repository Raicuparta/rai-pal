import { useCallback, useEffect, useRef } from "react";
import { useSetAtom } from "jotai";
import { commands, PROVIDER_IDS } from "@api/bindings";
import {
	loadingTasksAtom,
	gameDataAtom,
	modsAtom,
	gameDataVersionAtom,
} from "./use-data";
import { showAppNotification } from "@components/app-notifications";
import { useAppEvent } from "./use-app-event";
import { useThrottledCallback } from "@mantine/hooks";
import { useDataQuery } from "./use-data-query";
import { useCommandAtomData } from "./use-command-data";

export function useUpdateData(executeOnMount = false) {
	const setLoadingTasks = useSetAtom(loadingTasksAtom);
	const [gamesQuery] = useDataQuery();
	const totalLoadingTaskCount = useRef(0);
	const hasExecutedOnMount = useRef(false);

	const getGameIds = useCallback(
		() => commands.getGameIds(gamesQuery),
		[gamesQuery],
	);

	const updateGames = useCommandAtomData(getGameIds, gameDataAtom);

	const updateMods = useCommandAtomData(commands.getMods, modsAtom);

	const setGameDataVersion = useSetAtom(gameDataVersionAtom);

	useEffect(() => {
		if (!executeOnMount) return;
		updateGames();
		updateMods();
	}, [executeOnMount, updateGames, updateMods]);

	const throttledUpdateData = useThrottledCallback(async () => {
		await updateGames();
		setGameDataVersion((v) => v + 1);
		updateMods();
	}, 1000);

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
