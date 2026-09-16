import { showAppNotification } from "@components/app-notifications";
import { atom, useAtomValue, useSetAtom, useStore } from "jotai";
import { useCallback, useEffect } from "react";
import { commands } from "@api/bindings";
import { useAppEvent } from "./use-app-event";
import { modsAtom } from "./use-data";

export const runningModsAtom = atom<Record<string, true>>({});

export function useIsModRunning(modId: string) {
	const runningMods = useAtomValue(runningModsAtom);

	return Boolean(runningMods[modId]);
}

export function useRunningMods() {
	const setRunningMods = useSetAtom(runningModsAtom);
	const store = useStore();

	useEffect(() => {
		commands
			.getRunningMods()
			.then((infos) =>
				setRunningMods(
					Object.fromEntries(infos.map((info) => [info.modId, true as const])),
				),
			)
			.catch((error) => {
				showAppNotification(`Failed to get running mods: ${error}`, "error");
			});
	}, [setRunningMods]);

	useAppEvent(
		"modRunStateChanged",
		"running-mods",
		useCallback(
			(payload) => {
				setRunningMods((previous) => {
					const next = { ...previous };

					if (payload.running) {
						next[payload.mod_id] = true;
					} else {
						delete next[payload.mod_id];
					}

					return next;
				});

				if (!payload.running && payload.exit_code) {
					const modTitle =
						store.get(modsAtom)[payload.mod_id]?.title ?? payload.mod_id;

					showAppNotification(
						`Managed mod "${modTitle}" stopped unexpectedly (exit code ${payload.exit_code}).`,
						"error",
					);
				}
			},
			[setRunningMods, store],
		),
	);
}
