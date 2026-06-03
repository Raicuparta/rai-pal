import { Alert, Divider, Stack, Table } from "@mantine/core";
import { DbGame, commands } from "@api/bindings";
import { useCallback } from "react";
import { CommandButton } from "@components/command-button";
import { IconTrash } from "@tabler/icons-react";
import { GameModRow } from "./game-mod-row";
import { useLocalization } from "@hooks/use-localization";
import { useCommandData } from "@hooks/use-command-data";
import { MutedText } from "@components/muted-text";
import { GameModsData } from "@hooks/use-game-mods";

type Props = {
	readonly game: DbGame;
	readonly mods: GameModsData;
};

export function GameMods({ game, mods }: Props) {
	const t = useLocalization("gameModal");
	const getRemoteConfigs = useCallback(
		() => commands.getRemoteConfigs(game.providerId, game.gameId),
		[game],
	);
	const [remoteConfigs] = useCommandData(
		getRemoteConfigs,
		null,
		!game?.exePath,
	);

	if (mods.compatibleMods.length + mods.incompatibleMods.length === 0) {
		return null;
	}

	return (
		<>
			<Stack>
				{mods.compatibleMods.length > 0 && (
					<>
						<Divider label={t("gameModsLabel")} />
						{!game.exePath && (
							<Alert color="orange">{t("gameNotInstalledWarning")}</Alert>
						)}
						<Table
							highlightOnHover
							highlightOnHoverColor="dark.7"
						>
							<Table.Tbody>
								{mods.compatibleMods.map(({ mod, info }) => (
									<GameModRow
										key={mod.id}
										game={game}
										mod={mod}
										remoteConfigs={remoteConfigs}
										info={info}
									/>
								))}
							</Table.Tbody>
						</Table>
					</>
				)}
				{game.exePath && (
					<CommandButton
						confirmationText={t("uninstallAllModsConfirmation")}
						onClick={() =>
							commands.uninstallAllMods(game.providerId, game.gameId)
						}
						color="red"
						variant="light"
						leftSection={<IconTrash />}
					>
						{t("uninstallAllModsButton")}
					</CommandButton>
				)}
			</Stack>
			{mods.incompatibleMods.length > 0 && (
				<Stack>
					<Divider label={t("incompatibleGameModsLabel")} />
					<MutedText>{t("incompatibleGameModsDescription")}</MutedText>
					<Table>
						<Table.Tbody>
							{mods.incompatibleMods.map(({ mod, info }) => (
								<GameModRow
									key={mod.id}
									game={game}
									mod={mod}
									remoteConfigs={remoteConfigs}
									info={info}
									incompatible
								/>
							))}
						</Table.Tbody>
					</Table>
				</Stack>
			)}
		</>
	);
}
