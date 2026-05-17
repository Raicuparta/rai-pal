import { DbGame, commands } from "@api/bindings";
import { CommandButton } from "@components/command-button";
import { IconRefreshAlert } from "@tabler/icons-react";
import { UnifiedMod } from "@hooks/use-unified-mods";
import { useLocalization } from "@hooks/use-localization";

type Props = {
	readonly game: DbGame;
	readonly mod: UnifiedMod;
};

export function GameModUpdateButton({ game, mod }: Props) {
	const t = useLocalization("gameModRow");

	return (
		<CommandButton
			leftSection={<IconRefreshAlert />}
			color="green"
			variant="light"
			onClick={() =>
				commands.installMod(game.providerId, game.gameId, mod.merged.id)
			}
		>
			{t("updateMod")}
		</CommandButton>
	);
}
