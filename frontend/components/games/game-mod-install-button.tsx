import { DbGame, commands } from "@api/bindings";
import { IconCirclePlus } from "@tabler/icons-react";
import { UnifiedMod } from "@hooks/use-unified-mods";
import { useLocalization } from "@hooks/use-localization";
import { GameModActionButton } from "./game-mod-action-button";

type Props = {
	readonly game: DbGame;
	readonly mod: UnifiedMod;
};

export function GameModInstallButton({ game, mod }: Props) {
	const t = useLocalization("gameModRow");

	return (
		<GameModActionButton
			leftSection={<IconCirclePlus />}
			confirmationText={
				// TODO: translate
				"Attention: be careful when installing mods on multiplayer games! Anticheat can detect some mods and get you banned, even if the mods seem harmless."
			}
			confirmationSkipId="install-mod-confirm"
			onClick={() =>
				commands.installMod(game.providerId, game.gameId, mod.merged.id)
			}
		>
			{t("installMod")}
		</GameModActionButton>
	);
}
