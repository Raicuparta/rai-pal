import { useEffect } from "react";
import { event } from "@tauri-apps/api";
import { atom } from "jotai";
import { commands } from "@api/bindings";
import { dataSubscription } from "./use-data-subscription";
import { useUpdateData } from "./use-update-data";
import { useAsyncCommand } from "./use-async-command";

export const [installedGamesAtom, useInstalledGamesSubscription] =
	dataSubscription("SyncInstalledGames", commands.getInstalledGames, {});

export const [discoverGamesAtom, useDiscoverGamesSubscription] =
	dataSubscription("SyncDiscoverGames", commands.getDiscoverGames, []);

export const [modLoadersAtom, useModLoadersSubscription] = dataSubscription(
	"SyncMods",
	commands.getModLoaders,
	{},
);
export const [ownedGamesAtom, useOwnedGamesSubscription] = dataSubscription(
	"SyncOwnedGames",
	commands.getOwnedGames,
	[],
);

export const loadingAtom = atom<boolean>(false);

export function useData() {
	useInstalledGamesSubscription();
	useDiscoverGamesSubscription();
	useModLoadersSubscription();
	useOwnedGamesSubscription();

	const updateData = useUpdateData();

	const [executeAddGame] = useAsyncCommand(commands.addGame);

	useEffect(() => {
		const unlistenPromise = event.listen<string[]>(
			event.TauriEvent.WINDOW_FILE_DROP,
			(event) => {
				if (event.payload.length > 0) {
					executeAddGame(event.payload[0]);
				}
			},
		);

		return () => {
			unlistenPromise.then((unlisten) => unlisten());
		};
	}, [executeAddGame]);

	useEffect(() => {
		updateData();
	}, [updateData]);
}
