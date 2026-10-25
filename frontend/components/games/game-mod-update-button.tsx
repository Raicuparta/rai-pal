import { DbGame, GameMod, commands } from "@api/bindings";
import { CommandButton } from "@components/command-button";
import { IconRefreshAlert } from "@tabler/icons-react";
import { useLocalization } from "@hooks/use-localization";

type Props = {
	readonly game: DbGame;
	readonly mod: GameMod;
};

export function GameModUpdateButton({ game, mod }: Props) {
	const { t } = useLocalization("gameModRow");

	return (
		<CommandButton
			leftSection={<IconRefreshAlert />}
			color="green"
			variant="light"
			onClick={async () => {
				await commands.installMod(mod.id, game.providerId, game.gameId);

				commands.sendAnalyticsEvent("UpdateMod", {
					mod_id: mod.id,
					game: game.displayTitle,
				});
			}}
		>
			{t("updateMod")}
		</CommandButton>
	);
}
