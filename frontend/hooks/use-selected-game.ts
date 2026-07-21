import { useMemo, useState } from "react";
import { useAtomValue } from "jotai";
import { useAppEvent } from "./use-app-event";
import { modsAtom } from "./use-data";
import { commands, DbGame, GameMod, GameModInfo } from "@api/bindings";

export type GameModsPart = { mod: GameMod; info: GameModInfo };

export type GameModsData = {
	compatibleMods: GameModsPart[];
	hiddenMods: GameModsPart[];
	incompatibleMods: GameModsPart[];
};

export type SelectedGameData = {
	game: DbGame;
	modInfos: GameModInfo[];
};

export function useSelectedGame() {
	const [selected, setSelected] = useState<SelectedGameData | null>(null);
	const mods = useAtomValue(modsAtom);

	useAppEvent(
		"selectGame",
		"selected-game",
		(payload: SelectedGameData | null) => {
			setSelected(payload);
		},
	);

	useAppEvent(
		"refreshGame",
		"selected-game-refresh",
		([providerId, gameId]) => {
			setSelected((current) => {
				if (
					current?.game.providerId === providerId &&
					current.game.gameId === gameId
				) {
					commands.setSelectedGame(providerId, gameId);
				}
				return current;
			});
		},
	);

	const gameMods = useMemo<GameModsData | null>(() => {
		if (!selected?.modInfos) return null;

		const compatible: GameModsPart[] = [];
		const hidden: GameModsPart[] = [];
		const incompatible: GameModsPart[] = [];

		for (const info of selected.modInfos) {
			const mod = mods[info.modId];
			if (!mod) continue;

			if (info.compatible) {
				if (mod.hideFromGameModsList) {
					hidden.push({ mod, info });
				} else {
					compatible.push({ mod, info });
				}
			} else {
				incompatible.push({ mod, info });
			}
		}

		return {
			compatibleMods: compatible,
			hiddenMods: hidden,
			incompatibleMods: incompatible,
		};
	}, [selected, mods]);

	return {
		selectedGame: selected?.game ?? null,
		gameMods: selected ? gameMods : null,
	};
}
