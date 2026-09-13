import { commands, GameProviderId } from "@api/bindings";
import { CommandButton } from "@components/command-button";
import { useLocalization } from "@hooks/use-localization";
import { IconTrash } from "@tabler/icons-react";
import { useAsyncCommand } from "@hooks/use-async-command";

type Props = {
	readonly providerId: GameProviderId;
	readonly gameId: string;
};

export function RemoveGameButton(props: Props) {
	const { t } = useLocalization("gameModal");
	const [clearSelection] = useAsyncCommand(() =>
		commands.setSelectedGame(null, null),
	);
	const [refreshGames] = useAsyncCommand(commands.refreshGames);

	return (
		<CommandButton
			onClick={() => commands.removeGame(props.providerId, props.gameId)}
			confirmationText={t("removeGameConfirmation")}
			onSuccess={() => {
				clearSelection();
				refreshGames(props.providerId);
			}}
			leftSection={<IconTrash />}
		>
			{t("removeFromRaiPal")}
		</CommandButton>
	);
}
