import { DbGame, GameMod, commands } from "@api/bindings";
import { CommandButton } from "@components/command-button";
import { IconPlayerPlay } from "@tabler/icons-react";
import { useLocalization } from "@hooks/use-localization";

type Props = {
	readonly game: DbGame;
	readonly mod: GameMod;
};

export function GameModRunButton({ game, mod }: Props) {
	const t = useLocalization("gameModRow");

	return (
		<CommandButton
			leftSection={<IconPlayerPlay />}
			onClick={() => commands.runMod(game.providerId, game.gameId, mod.id)}
		>
			{t("runMod")}
		</CommandButton>
	);
}
