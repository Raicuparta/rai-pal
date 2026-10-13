import { useCallback, useEffect, useRef } from "react";
import { useSetAtom } from "jotai";
import { commands, PROVIDER_IDS } from "@api/bindings";
import {
	gameDataAtom,
	modsAtom,
	modFamiliesAtom,
	gameDataVersionAtom,
} from "./use-data";
import { useAppEvent } from "./use-app-event";
import { useDebouncedCallback } from "@mantine/hooks";
import { useDataQuery } from "./use-data-query";
import { useCommandAtomData } from "./use-command-data";
import { useRefreshTask } from "./use-refresh-tasks";

export function useUpdateData(executeOnMount = false) {
	const runRefreshTask = useRefreshTask();
	const [gamesQuery] = useDataQuery();
	const hasExecutedOnMount = useRef(false);

	const getGameIds = useCallback(
		() => commands.getGameIds(gamesQuery),
		[gamesQuery],
	);

	const updateGames = useCommandAtomData(getGameIds, gameDataAtom);

	const updateMods = useCommandAtomData(commands.getMods, modsAtom);

	const updateModFamilies = useCommandAtomData(
		commands.getModFamilies,
		modFamiliesAtom,
	);

	const setGameDataVersion = useSetAtom(gameDataVersionAtom);

	useEffect(() => {
		if (!executeOnMount) return;
		updateGames();
		updateMods();
		updateModFamilies();
	}, [executeOnMount, updateGames, updateMods, updateModFamilies]);

	const debouncedUpdateData = useDebouncedCallback(async () => {
		// Fetch games and mods concurrently so a slow games fetch doesn't delay
		// the mods list update. Debouncing (trailing-only) means this runs once
		// after the database-write burst settles, so it always reads final data.
		await Promise.all([updateGames(), updateMods(), updateModFamilies()]);
		setGameDataVersion((v) => v + 1);
	}, 500);

	useAppEvent("appDatabaseChanged", "update-data", debouncedUpdateData);

	const updateAppData = useCallback(async () => {
		for (const providerId of PROVIDER_IDS) {
			runRefreshTask(providerId, commands.refreshGames(providerId));
		}

		runRefreshTask("mods", commands.refreshMods());
		runRefreshTask("remote data", commands.refreshRemoteGames());
	}, [runRefreshTask]);

	useEffect(() => {
		if (executeOnMount && !hasExecutedOnMount.current) {
			updateAppData();
			hasExecutedOnMount.current = true;
		}
	}, [executeOnMount, updateAppData]);

	return updateAppData;
}
