import { DbGame, GameMod, commands } from "@api/bindings";
import { CommandButton } from "@components/command-button";
import { IconPlayerPlay, IconPlayerStop } from "@tabler/icons-react";
import { useLocalization } from "@hooks/use-localization";
import { useIsModRunning } from "@hooks/use-running-mods";

type Props = {
	readonly game: DbGame;
	readonly mod: GameMod;
};

export function GameModRunButton({ game, mod }: Props) {
	const { t } = useLocalization("gameModRow");
	const isRunning = useIsModRunning(mod.id, game.providerId, game.gameId);

	if (isRunning) {
		return (
			<CommandButton
				leftSection={<IconPlayerStop />}
				onClick={async () => {
					await commands.stopMod(mod.id, game.providerId, game.gameId);
				}}
			>
				{t("stopMod")}
			</CommandButton>
		);
	}

	return (
		<CommandButton
			leftSection={<IconPlayerPlay />}
			onClick={async () => {
				await commands.runMod(mod.id, game.providerId, game.gameId);

				commands.sendAnalyticsEvent("RunMod", {
					mod_id: mod.id,
					game: game.displayTitle,
				});
			}}
		>
			{t("runMod")}
		</CommandButton>
	);
}
