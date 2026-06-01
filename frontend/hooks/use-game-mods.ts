import { commands, GameMod, GameModInfo, GameProviderId } from "@api/bindings";
import { useAppEvent } from "./use-app-event";
import { useCommandData } from "./use-command-data";
import { useCallback, useMemo } from "react";
import { useAtomValue } from "jotai";
import { modsAtom } from "./use-data";

type GameModsPart = { mod: GameMod; info: GameModInfo };
export type GameModsData = {
	compatibleMods: GameModsPart[];
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

	const result = useMemo(() => {
		const compatibleMods: { mod: GameMod; info: GameModInfo }[] = [];
		const incompatibleMods: { mod: GameMod; info: GameModInfo }[] = [];

		if (!gameModInfos) return null;

		for (const info of gameModInfos) {
			const mod = mods[info.modId];
			if (!mod) continue;

			if (info.compatible) {
				// Conditions that determine whether a mod is worthy of showing in the mods list for a specific game.
				// If no install nor run, then there's nothing to do.
				// If nothing to download, then likely just an extra dependency (might change in the future dunno).
				if ((mod.install || mod.runForGame) && mod.download) {
					compatibleMods.push({ mod, info });
				}
			} else {
				incompatibleMods.push({ mod, info });
			}
		}

		return { compatibleMods, incompatibleMods };
	}, [gameModInfos, mods]);

	return result;
}
