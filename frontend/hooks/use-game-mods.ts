import { commands, GameMod, GameModInfo, GameProviderId } from "@api/bindings";
import { useAppEvent } from "./use-app-event";
import { useCommandData } from "./use-command-data";
import { useCallback, useMemo } from "react";
import { useAtomValue } from "jotai";
import { modsAtom } from "./use-data";

type GameModsPart = { mod: GameMod; info: GameModInfo };
export type GameModsData = {
	compatibleMods: GameModsPart[];
	hiddenMods: GameModsPart[];
	incompatibleMods: GameModsPart[];
};

export function useGameMods(
	providerId?: GameProviderId,
	gameId?: string,
): GameModsData | null {
	const mods = useAtomValue(modsAtom);
	const getGameMods = useCallback(
		async () =>
			providerId && gameId ? commands.getGameMods(providerId, gameId) : null,
		[providerId, gameId],
	);
	const [gameModInfos, updateGameModInfos] = useCommandData(getGameMods, null);

	useAppEvent(
		"refreshGame",
		`installed-mods-${providerId}:${gameId}`,
		([refreshedProviderId, refreshedGameId]) => {
			if (refreshedProviderId !== providerId || refreshedGameId !== gameId)
				return;

			updateGameModInfos();
		},
	);

	useAppEvent(
		"appDatabaseChanged",
		`app-data-mods-${providerId}:${gameId}`,
		() => {
			if (!providerId || !gameId) return;

			return updateGameModInfos();
		},
	);

	const result = useMemo(() => {
		const compatibleMods: { mod: GameMod; info: GameModInfo }[] = [];
		const incompatibleMods: { mod: GameMod; info: GameModInfo }[] = [];
		const hiddenMods: { mod: GameMod; info: GameModInfo }[] = [];

		if (!gameModInfos) return null;

		for (const info of gameModInfos) {
			const mod = mods[info.modId];
			if (!mod) continue;

			if (info.compatible) {
				if (mod.hideFromGameModsList) {
					hiddenMods.push({ mod, info });
				} else {
					compatibleMods.push({ mod, info });
				}
			} else {
				incompatibleMods.push({ mod, info });
			}
		}

		return { compatibleMods, hiddenMods, incompatibleMods };
	}, [gameModInfos, mods]);

	return result;
}
