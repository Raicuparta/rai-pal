import { DbGame, GameMod, commands } from "@api/bindings";
import { CommandButton } from "@components/command-button";
import { IconRefreshAlert } from "@tabler/icons-react";
import { useLocalization } from "@hooks/use-localization";

type Props = {
	readonly game: DbGame;
	readonly mod: GameMod;
	readonly remoteConfigFile?: string;
};

export function GameModUpdateButton({ game, mod, remoteConfigFile }: Props) {
	const t = useLocalization("gameModRow");

	return (
		<CommandButton
			leftSection={<IconRefreshAlert />}
			color="green"
			variant="light"
			onClick={async () => {
				if (remoteConfigFile) {
					await commands.downloadRemoteConfig(
						game.providerId,
						game.gameId,
						mod.id,
						remoteConfigFile,
						false,
					);
				}

				await commands.installMod(mod.id, game.providerId, game.gameId);

				commands.sendAnalyticsEvent("update_mod", {
					mod_id: mod.id,
					game: game.displayTitle,
				});
			}}
		>
			{t("updateMod")}
		</CommandButton>
	);
}
