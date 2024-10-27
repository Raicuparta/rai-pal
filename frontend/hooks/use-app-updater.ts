import { useEffect, useRef } from "react";

import { check as checkUpdate } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";
import { ask } from "@tauri-apps/plugin-dialog";
import { showAppNotification } from "@components/app-notifications";

const CHECK_INTERVAL_MILLISECONDS = 120000;

export function useAppUpdater() {
	// Use this ID as a way to prevent multiple checks at once.
	const updateCheckId = useRef(0);

	useEffect(() => {
		let shouldSkipUpdate = false;

		function triggerUpdateCheck() {
			// Increment the ID and use it for this check.
			updateCheckId.current++;
			const currentCheckId = updateCheckId.current;

			checkUpdate()
				.then(async (update) => {
					if (currentCheckId !== updateCheckId.current) {
						// If the IDs are different, that means a new check has started in the meantime.
						return;
					}

					if (!update?.available || shouldSkipUpdate) return;

					console.log(
						`Received update ${update.version}, ${update.date}, ${update.body}`,
					);

					// Skip any checks that happen while this one is open.
					shouldSkipUpdate = true;

					const userAcceptedUpdate = await ask(
						update.body || "(no changelog)",
						{
							kind: "info",
							cancelLabel: "Ignore (won't ask again until you restart Rai Pal)",
							okLabel: "Update now",
							title: `Rai Pal Update ${update.version}`,
						},
					);

					shouldSkipUpdate = false;

					if (!userAcceptedUpdate) {
						// If the user says no, let's not bother them any longer during this session.
						shouldSkipUpdate = true;
						clearInterval(interval);

						return;
					}

					await update.downloadAndInstall();
					await relaunch();
				})
				.catch((error) => {
					showAppNotification(`Failed to get app updates: ${error}`, "error");
				});
		}

		// Initial check on mount.
		triggerUpdateCheck();

		// Subsequent checks every so often.
		const interval = setInterval(
			triggerUpdateCheck,
			CHECK_INTERVAL_MILLISECONDS,
		);

		return () => {
			clearInterval(interval);
		};
	}, []);
}
