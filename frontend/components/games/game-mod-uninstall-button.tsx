import { DbGame, commands } from "@api/bindings";
import { CommandButton } from "@components/command-button";
import { IconTrash } from "@tabler/icons-react";
import { UnifiedMod } from "@hooks/use-unified-mods";
import { useLocalization } from "@hooks/use-localization";

type Props = {
	readonly game: DbGame;
	readonly mod: UnifiedMod;
};

export function GameModUninstallButton({ game, mod }: Props) {
	const t = useLocalization("gameModRow");

	return (
		<CommandButton
			leftSection={<IconTrash />}
			color="red"
			variant="light"
			onClick={() =>
				commands.uninstallMod(game.providerId, game.gameId, mod.id)
			}
		>
			{t("uninstallMod")}
		</CommandButton>
	);
}
