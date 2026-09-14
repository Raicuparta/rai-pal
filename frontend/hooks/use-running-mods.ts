import { showAppNotification } from "@components/app-notifications";
import { atom, useAtomValue, useSetAtom } from "jotai";
import { useCallback, useEffect } from "react";
import { GameProviderId, commands } from "@api/bindings";
import { useAppEvent } from "./use-app-event";

export const runningModsAtom = atom<Record<string, true>>({});

export function runningModKey(
	modId: string,
	providerId: GameProviderId | null,
	gameId: string | null,
) {
	return `${modId}|${providerId}|${gameId}`;
}

export function useIsModRunning(
	modId: string,
	providerId: GameProviderId | null,
	gameId: string | null,
) {
	const runningMods = useAtomValue(runningModsAtom);

	return Boolean(runningMods[runningModKey(modId, providerId, gameId)]);
}

export function useRunningMods() {
	const setRunningMods = useSetAtom(runningModsAtom);

	useEffect(() => {
		commands
			.getRunningMods()
			.then((infos) =>
				setRunningMods(
					Object.fromEntries(
						infos.map((info) => [
							runningModKey(info.modId, info.providerId, info.gameId),
							true as const,
						]),
					),
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
				const key = runningModKey(
					payload.mod_id,
					payload.provider_id,
					payload.game_id,
				);

				setRunningMods((previous) => {
					const next = { ...previous };

					if (payload.running) {
						next[key] = true;
					} else {
						delete next[key];
					}

					return next;
				});
			},
			[setRunningMods],
		),
	);
}
