import { useEffect, useRef } from "react";

import {
	checkUpdate,
	installUpdate,
	onUpdaterEvent,
} from "@tauri-apps/api/updater";
import { relaunch } from "@tauri-apps/api/process";
import { showAppNotification } from "@components/app-notifications";
import { dialog } from "@tauri-apps/api";

const checkIntervalMilliseconds = 120000;

export function useAppUpdater() {
	// Use this ID as a way to prevent multiple checks at once.
	const updateCheckId = useRef(0);

	useEffect(() => {
		const unlistenPromise = onUpdaterEvent(({ error, status }) => {
			(error ? console.error : console.log)(error || status);
		});

		let shouldSkipUpdate = false;

		function triggerUpdateCheck() {
			// Increment the ID and use it for this check.
			updateCheckId.current++;
			const currentCheckId = updateCheckId.current;

			checkUpdate()
				.then(async ({ shouldUpdate, manifest }) => {
					if (!manifest) {
						throw new Error("Update manifest not present.");
					}

					if (currentCheckId !== updateCheckId.current) {
						// If the IDs are different, that means a new check has started in the meantime.
						return;
					}

					if (!shouldUpdate || shouldSkipUpdate) return;

					console.log(
						`Received update ${manifest.version}, ${manifest.date}, ${manifest.body}`,
					);

					// Skip any checks that happen while this one is open.
					shouldSkipUpdate = true;

					const userAcceptedUpdate = await dialog.ask(manifest.body, {
						type: "info",
						cancelLabel: "Ignore (won't ask again until you restart Rai Pal)",
						okLabel: "Update now",
						title: `Rai Pal Update ${manifest.version}`,
					});

					shouldSkipUpdate = false;

					if (!userAcceptedUpdate) {
						// If the user says no, let's not bother them any longer during this session.

						shouldSkipUpdate = true;
						clearInterval(interval);

						return;
					}

					// Install the update. This will also restart the app on Windows!
					await installUpdate();

					// On macOS and Linux we need to restart manually.
					await relaunch();
				})
				.catch((error) => {
					showAppNotification(`Failed to get app updates: ${error}`, "error");
				});
		}

		// Initial check on mount.
		triggerUpdateCheck();

		// Subsequent checks every so often.
		const interval = setInterval(triggerUpdateCheck, checkIntervalMilliseconds);

		return () => {
			unlistenPromise.then((unlisten) => unlisten());
			clearInterval(interval);
		};
	}, []);
}
