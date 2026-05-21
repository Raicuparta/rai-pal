import { DbGame, commands } from "@api/bindings";
import { CommandButton } from "@components/command-button";
import { IconRefreshAlert } from "@tabler/icons-react";
import { UnifiedMod } from "@hooks/use-unified-mods";
import { useLocalization } from "@hooks/use-localization";

type Props = {
	readonly game: DbGame;
	readonly mod: UnifiedMod;
	readonly isLocalModOutdated: boolean;
};

export function GameModUpdateButton({ game, mod, isLocalModOutdated }: Props) {
	const t = useLocalization("gameModRow");

	return (
		<CommandButton
			leftSection={<IconRefreshAlert />}
			color="green"
			variant="light"
			onClick={async () => {
				if (isLocalModOutdated) {
					await commands.downloadMod(mod.id);
				}
				await commands.installMod(game.providerId, game.gameId, mod.id);
			}}
		>
			{t("updateMod")}
		</CommandButton>
	);
}
