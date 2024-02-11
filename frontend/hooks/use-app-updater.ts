import { emit } from "@tauri-apps/api/event";
import { useEffect } from "react";

import {
	checkUpdate,
	installUpdate,
	onUpdaterEvent,
} from "@tauri-apps/api/updater";
import { relaunch } from "@tauri-apps/api/process";
import { showAppNotification } from "@components/app-notifications";
import { dialog } from "@tauri-apps/api";

const checkFrequencyMilliseconds = 3.6e6;

export function useAppUpdater() {
	useEffect(() => {
		const unlistenPromise = onUpdaterEvent(({ error, status }) => {
			(error ? console.error : console.log)(error || status);
		});

		checkUpdate()
			.then(async ({ shouldUpdate, manifest }) => {
				if (shouldUpdate) {
					console.log(
						`Installing update ${manifest?.version}, ${manifest?.date}, ${manifest?.body}`,
					);

					const shouldUpdate = await dialog.ask(
						`A new Rai Pal update is available.\n\nIf you skip the update for now, you'll be prompted again once you restart Rai Pal.\n\nChanges in version ${manifest?.version}:\n\n${manifest?.body}`,
						{
							type: "info",
							cancelLabel: "No thanks",
							okLabel: "Update now",
							title: "Rai Pal Update",
						},
					);

					if (!shouldUpdate) {
						return;
					}

					// Install the update. This will also restart the app on Windows!
					await installUpdate();

					// On macOS and Linux we need to restart manually.
					await relaunch();
				}
			})
			.catch((error) => {
				showAppNotification(`Failed to get app updates: ${error}`, "error");
			});

		const interval = setInterval(() => {
			if (!document.hasFocus()) return;
			emit("tauri://update");
		}, checkFrequencyMilliseconds);

		return () => {
			unlistenPromise.then((unlisten) => unlisten());
			clearInterval(interval);
		};
	}, []);
}
