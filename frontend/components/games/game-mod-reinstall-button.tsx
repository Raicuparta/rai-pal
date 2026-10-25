import { DbGame, GameMod, commands } from "@api/bindings";
import { CommandButton } from "@components/command-button";
import { IconRefresh } from "@tabler/icons-react";
import { useLocalization } from "@hooks/use-localization";

type Props = {
	readonly game: DbGame;
	readonly mod: GameMod;
};

export function GameModReinstallButton({ game, mod }: Props) {
	const { t } = useLocalization("gameModRow");

	return (
		<CommandButton
			leftSection={<IconRefresh />}
			onClick={async () => {
				await commands.installMod(mod.id, game.providerId, game.gameId);

				commands.sendAnalyticsEvent("InstallMod", {
					mod_id: mod.id,
					game: game.displayTitle,
				});
			}}
		>
			{t("reinstallMod")}
		</CommandButton>
	);
}
