import { commands, ProviderId } from "@api/bindings";
import { CommandButton } from "@components/command-button";
import { useLocalization } from "@hooks/use-localization";
import { IconTrash } from "@tabler/icons-react";
import { useSetAtom } from "jotai";
import { selectedGameAtom } from "./games-state";
import { useAsyncCommand } from "@hooks/use-async-command";

type Props = {
	readonly providerId: ProviderId;
	readonly gameId: string;
};

export function RemoveGameButton(props: Props) {
	const t = useLocalization("gameModal");
	const setSelectedGame = useSetAtom(selectedGameAtom);
	const [refreshGames] = useAsyncCommand(commands.refreshGames);

	return (
		<CommandButton
			onClick={() => commands.removeGame(props.providerId, props.gameId)}
			confirmationText={t("removeGameConfirmation")}
			onSuccess={() => {
				setSelectedGame((previous) => {
					if (
						previous?.providerId === props.providerId &&
						previous?.gameId === props.gameId
					) {
						return null;
					}
					return previous;
				});

				refreshGames(props.providerId);
			}}
			leftSection={<IconTrash />}
		>
			{t("removeFromRaiPal")}
		</CommandButton>
	);
}
