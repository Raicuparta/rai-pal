import { emit, listen } from "@tauri-apps/api/event";
import { useEffect } from "react";

const checkFrequencyMilliseconds = 3.6e6;

export function useAppUpdates() {
	useEffect(() => {
		console.log("listen");
		const unlistenPromise = listen("tauri://update-available", () => {
			// If an update is available, the update prompt will show,
			// and we don't want to keep checking for updates,
			// since that will stack multiple prompts.
			console.log("update available");
			clearInterval(interval);
		});

		const interval = setInterval(() => {
			console.log("update emit");
			emit("tauri://update");
		}, checkFrequencyMilliseconds);

		return () => {
			console.log("unlisten");
			unlistenPromise.then((unlisten) => unlisten());
			clearInterval(interval);
		};
	}, []);
}
