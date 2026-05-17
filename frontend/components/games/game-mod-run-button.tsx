import { DbGame, commands } from "@api/bindings";
import { CommandButton } from "@components/command-button";
import { IconPlayerPlay } from "@tabler/icons-react";
import { UnifiedMod } from "@hooks/use-unified-mods";
import { useLocalization } from "@hooks/use-localization";

type Props = {
	readonly game: DbGame;
	readonly mod: UnifiedMod;
};

export function GameModRunButton({ game, mod }: Props) {
	const t = useLocalization("gameModRow");

	return (
		<CommandButton
			leftSection={<IconPlayerPlay />}
			onClick={() =>
				commands.runMod(game.providerId, game.gameId, mod.merged.id)
			}
		>
			{t("runMod")}
		</CommandButton>
	);
}
