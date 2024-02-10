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

export function useAppUpdates() {
	useEffect(() => {
		const unlistenPromise = onUpdaterEvent(({ error, status }) => {
			showAppNotification(error || status, error ? "error" : "info");
		});

		checkUpdate()
			.then(async ({ shouldUpdate, manifest }) => {
				if (shouldUpdate) {
					// You could show a dialog asking the user if they want to install the update here.
					console.log(
						`Installing update ${manifest?.version}, ${manifest?.date}, ${manifest?.body}`,
					);

					const shouldUpdate = await dialog.ask("Wanna update this bad boy?", {
						type: "info",
						cancelLabel: "No thanks",
						okLabel: "Update now",
						title: "Hey",
					});

					if (!shouldUpdate) {
						return;
					}

					// Install the update. This will also restart the app on Windows!
					await installUpdate();

					// On macOS and Linux you will need to restart the app manually.
					// You could use this step to display another confirmation dialog.
					await relaunch();
				}
			})
			.catch((error) => {
				showAppNotification(error, "error");
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
