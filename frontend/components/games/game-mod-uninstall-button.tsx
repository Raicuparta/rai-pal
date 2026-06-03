import { DbGame, GameMod, GameModInfo, commands } from "@api/bindings";
import { CommandButton } from "@components/command-button";
import { IconTrash } from "@tabler/icons-react";
import { useLocalization } from "@hooks/use-localization";
import { Tooltip } from "@mantine/core";

type Props = {
	readonly game: DbGame;
	readonly mod: GameMod;
	readonly modInfo?: GameModInfo;
};

export function GameModUninstallButton({ game, mod, modInfo }: Props) {
	const t = useLocalization("gameModRow");

	return (
		<Tooltip
			disabled={!modInfo?.hasInstalledDependants}
			label={t("cantUninstallModWithDependants")}
			withinPortal
		>
			<CommandButton
				leftSection={<IconTrash />}
				color="red"
				variant="light"
				disabled={modInfo?.hasInstalledDependants}
				onClick={() =>
					commands.uninstallMod(game.providerId, game.gameId, mod.id)
				}
			>
				{t("uninstallMod")}
			</CommandButton>
		</Tooltip>
	);
}
