import { commands } from "@api/bindings";
import { useAsyncCommand } from "./use-async-command";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import { useEffect } from "react";

export function useGameDropEvent() {
	const [executeAddGame] = useAsyncCommand(commands.addGame);

	useEffect(() => {
		const unlistenPromise = getCurrentWebview().onDragDropEvent((event) => {
			if (event.payload.type === "drop") {
				const path = event.payload.paths[0];
				if (path) {
					executeAddGame(path);
				}
			}
		});

		return () => {
			unlistenPromise.then((unlisten) => unlisten());
		};
	}, [executeAddGame]);
}
