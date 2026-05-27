import { commands, GameModInfo, ProviderId } from "@api/bindings";
import { useAppEvent } from "./use-app-event";
import { useCommandData } from "./use-command-data";
import { useCallback, useMemo } from "react";
import { useUnifiedMods, UnifiedMod } from "./use-unified-mods";

type GameModsPart = { mod: UnifiedMod; info: GameModInfo };
export type GameModsData = {
	compatibleMods: GameModsPart[];
	incompatibleMods: GameModsPart[];
};

export function useGameMods(
	providerId?: ProviderId,
	gameId?: string,
): GameModsData | null {
	const mods = useUnifiedMods();
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
		const compatibleMods: { mod: UnifiedMod; info: GameModInfo }[] = [];
		const incompatibleMods: { mod: UnifiedMod; info: GameModInfo }[] = [];

		if (!gameModInfos) return null;

		for (const info of gameModInfos) {
			const mod = mods[info.modId];
			if (!mod) continue;

			if ((info.compatible && mod.install) || mod.runForGame) {
				compatibleMods.push({ mod, info });
			} else {
				incompatibleMods.push({ mod, info });
			}
		}

		return { compatibleMods, incompatibleMods };
	}, [gameModInfos, mods]);

	return result;
}
