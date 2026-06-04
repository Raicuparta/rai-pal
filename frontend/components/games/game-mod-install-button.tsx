import { DbGame, GameMod, commands } from "@api/bindings";
import { IconCirclePlus } from "@tabler/icons-react";
import { useLocalization } from "@hooks/use-localization";
import { GameModActionButton } from "./game-mod-action-button";

type Props = {
	readonly game: DbGame;
	readonly mod: GameMod;
	readonly remoteConfigFile?: string;
};

export function GameModInstallButton({ game, mod, remoteConfigFile }: Props) {
	const t = useLocalization("gameModRow");

	return (
		<GameModActionButton
			leftSection={<IconCirclePlus />}
			confirmationText={
				// TODO: translate
				"Attention: be careful when installing mods on multiplayer games! Anticheat can detect some mods and get you banned, even if the mods seem harmless."
			}
			confirmationSkipId="install-mod-confirm"
			onClick={async () => {
				commands.sendAnalyticsEvent("install_mod", {
					game: game.displayTitle,
					modId: mod.id,
				});

				if (remoteConfigFile) {
					await commands.downloadRemoteConfig(
						game.providerId,
						game.gameId,
						mod.id,
						remoteConfigFile,
						false,
					);
				}
				await commands.installMod(game.providerId, game.gameId, mod.id);
			}}
		>
			{t("installMod")}
		</GameModActionButton>
	);
}
