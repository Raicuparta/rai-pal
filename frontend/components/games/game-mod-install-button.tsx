import { DbGame, GameMod, commands } from "@api/bindings";
import { IconCirclePlus } from "@tabler/icons-react";
import { useLocalization } from "@hooks/use-localization";
import { GameModActionButton } from "./game-mod-action-button";

type Props = {
	readonly game: DbGame;
	readonly mod: GameMod;
};

export function GameModInstallButton({ game, mod }: Props) {
	const { t } = useLocalization("gameModRow");

	return (
		<GameModActionButton
			leftSection={<IconCirclePlus />}
			confirmationText={t("installModAnticheatWarning")}
			confirmationSkipId="install-mod-confirm"
			onClick={async () => {
				await commands.installMod(mod.id, game.providerId, game.gameId);
				commands.sendAnalyticsEvent("InstallMod", {
					mod_id: mod.id,
					game: game.displayTitle,
				});
			}}
		>
			{t("installMod")}
		</GameModActionButton>
	);
}
