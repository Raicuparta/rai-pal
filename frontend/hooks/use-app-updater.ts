import { useCallback, useEffect, useRef, useState } from "react";

import { showAppNotification } from "@components/app-notifications";
import { check as checkUpdate, Update } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";

const CHECK_INTERVAL_MILLISECONDS = 120000;

export type AvailableUpdate = {
	readonly version: string;
	readonly body?: string;
};

export function useAppUpdater() {
	// Use this ID as a way to prevent multiple checks at once.
	const updateCheckId = useRef(0);
	const pendingUpdate = useRef<Update | null>(null);
	const interval = useRef<ReturnType<typeof setInterval> | undefined>(
		undefined,
	);
	const [availableUpdate, setAvailableUpdate] =
		useState<AvailableUpdate | null>(null);

	useEffect(() => {
		function triggerUpdateCheck() {
			// Increment the ID and use it for this check.
			updateCheckId.current++;
			const currentCheckId = updateCheckId.current;

			checkUpdate()
				.then((update) => {
					if (currentCheckId !== updateCheckId.current) {
						// If the IDs are different, that means a new check has started in the meantime.
						return;
					}

					if (!update?.available || pendingUpdate.current) return;

					console.log(
						`Received update ${update.version}, ${update.date}, ${update.body}`,
					);

					pendingUpdate.current = update;
					setAvailableUpdate({
						version: update.version,
						body: update.body,
					});
				})
				.catch((error) => {
					console.error(`Failed to get app updates: ${error}`, "error");
				});
		}

		// Initial check on mount.
		triggerUpdateCheck();

		// Subsequent checks every so often.
		interval.current = setInterval(
			triggerUpdateCheck,
			CHECK_INTERVAL_MILLISECONDS,
		);

		return () => {
			clearInterval(interval.current);
		};
	}, []);

	const installUpdate = useCallback(async () => {
		const update = pendingUpdate.current;
		if (!update) return;
		try {
			console.log(`Downloading and installing update ${update.version}`);
			await update.downloadAndInstall();
			console.log("Update installed, relaunching Rai Pal...");
			await relaunch();
		} catch (error) {
			showAppNotification(`Failed to install app update: ${error}`, "error");
		}
	}, []);

	const ignoreUpdate = useCallback(() => {
		// If the user says no, let's not bother them any longer during this session.
		pendingUpdate.current = null;
		setAvailableUpdate(null);
		clearInterval(interval.current);
	}, []);

	return { availableUpdate, installUpdate, ignoreUpdate };
}
