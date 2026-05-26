import { Alert, Divider, Stack, Table } from "@mantine/core";
import { DbGame, GameModInfo, commands } from "@api/bindings";
import { useCallback, useMemo } from "react";
import { CommandButton } from "@components/command-button";
import { IconTrash } from "@tabler/icons-react";
import { UnifiedMod, useUnifiedMods } from "@hooks/use-unified-mods";
import { GameModRow } from "./game-mod-row";
import { useLocalization } from "@hooks/use-localization";
import { useCommandData } from "@hooks/use-command-data";
import { useAppEvent } from "@hooks/use-app-event";
import { MutedText } from "@components/muted-text";

type Props = {
	readonly game: DbGame;
};

const defaultModsInfo: GameModInfo[] = [];

export function GameMods({ game }: Props) {
	const t = useLocalization("gameModal");
	const mods = useUnifiedMods();
	const getGameMods = useCallback(
		() => commands.getGameMods(game.providerId, game.gameId),
		[game],
	);
	const [modsInfo, updateModsInfo] = useCommandData(
		getGameMods,
		defaultModsInfo,
	);
	const getRemoteConfigs = useCallback(
		() => commands.getRemoteConfigs(game.providerId, game.gameId),
		[game],
	);
	const [remoteConfigs] = useCommandData(
		getRemoteConfigs,
		null,
		!game?.exePath,
	);

	useAppEvent(
		"refreshGame",
		`installed-mods-${game.providerId}:${game.gameId}`,
		([refreshedProviderId, refreshedGameId]) => {
			if (
				refreshedProviderId !== game.providerId ||
				refreshedGameId !== game.gameId
			)
				return;
			updateModsInfo();
		},
	);

	const { compatibleMods, incompatibleMods } = useMemo(() => {
		const compatibleMods: { mod: UnifiedMod; info: GameModInfo }[] = [];
		const incompatibleMods: { mod: UnifiedMod; info: GameModInfo }[] = [];

		for (const info of modsInfo) {
			const mod = mods[info.modId];
			if (!mod) continue;

			if (info.compatible) {
				compatibleMods.push({ mod, info });
			} else {
				incompatibleMods.push({ mod, info });
			}
		}

		return { compatibleMods, incompatibleMods };
	}, [modsInfo, mods]);

	if (compatibleMods.length + incompatibleMods.length === 0) {
		return null;
	}

	return (
		<>
			<Stack>
				{compatibleMods.length > 0 && (
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
								{compatibleMods.map(({ mod, info }) => (
									<GameModRow
										key={mod.id}
										game={game}
										mod={mod}
										remoteConfigs={remoteConfigs}
										installedMod={
											info.installedVersion
												? {
														latestVersion: {
															id: info.installedVersion,
															url: "",
														},
														hash: info.installedHash,
													}
												: undefined
										}
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
			{incompatibleMods.length > 0 && (
				<Stack>
					<Divider label={t("incompatibleGameModsLabel")} />
					<MutedText>{t("incompatibleGameModsDescription")}</MutedText>
					<Table>
						<Table.Tbody>
							{incompatibleMods.map(({ mod, info }) => (
								<GameModRow
									key={mod.id}
									game={game}
									mod={mod}
									remoteConfigs={remoteConfigs}
									installedMod={
										info.installedVersion
											? {
													latestVersion: {
														id: info.installedVersion,
														url: "",
													},
													hash: info.installedHash,
												}
											: undefined
									}
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
