import { commands, events } from "@api/bindings";
import { useAsyncCommand } from "./use-async-command";
import { useAppEvent } from "./use-app-event";
import { useSetAtom } from "jotai";
import { installedGamesAtom } from "./use-data";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import { useEffect } from "react";

export function useGameDropEvent() {
	const [executeAddGame] = useAsyncCommand(commands.addGame);
	const setInstalledGames = useSetAtom(installedGamesAtom);

	// TODO: might not need this once I fix cache invalidation.
	useAppEvent(events.gameRemoved, (gameId) => {
		setInstalledGames((previousInstalledGames) => {
			// Mutating the inner value because we're doing some cursed thing for performance.
			delete previousInstalledGames[gameId];

			// But creating a new object for the outer structure, to count as a new reference.
			return {
				...previousInstalledGames,
			};
		});
	});

	useEffect(() => {
		const unlistenPromise = getCurrentWebview().onDragDropEvent((event) => {
			if (event.payload.type === "drop") {
				executeAddGame(event.payload.paths[0]);
			}
		});

		return () => {
			unlistenPromise.then((unlisten) => unlisten());
		};
	}, [executeAddGame]);
}
