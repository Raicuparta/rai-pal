import { Alert, Box, Divider, Stack, Table } from "@mantine/core";
import { DbGame, RemoteConfigs, commands } from "@api/bindings";
import { ReactNode, useCallback } from "react";
import { CommandButton } from "@components/command-button";
import { IconTrash } from "@tabler/icons-react";
import { GameModRow } from "./game-mod-row";
import { useLocalization } from "@hooks/use-localization";
import { useCommandData } from "@hooks/use-command-data";
import { MutedText } from "@components/muted-text";
import { GameModsData, GameModsPart } from "@hooks/use-selected-game";

type Props = {
	readonly game: DbGame;
	readonly mods: GameModsData;
};

type GameModsTableProps = {
	readonly game: DbGame;
	readonly mods: GameModsPart[];
	readonly remoteConfigs: RemoteConfigs | null;
	readonly incompatible?: boolean;
	readonly highlightOnHover?: boolean;
	readonly header?: ReactNode;
};

function GameModsTable({
	game,
	mods,
	remoteConfigs,
	incompatible,
	header,
}: GameModsTableProps) {
	return (
		<Table bg="var(--background-dark)">
			<Table.Tbody>
				{header}
				{mods.map(({ mod, info }) => (
					<GameModRow
						key={mod.id}
						game={game}
						mod={mod}
						remoteConfigs={remoteConfigs}
						info={info}
						incompatible={incompatible}
					/>
				))}
			</Table.Tbody>
		</Table>
	);
}

export function GameMods({ game, mods }: Props) {
	const { t } = useLocalization("gameModal");
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

	const installedMods = mods.compatibleMods.filter(
		({ info }) => info.installedHash,
	);
	const notInstalledMods = mods.compatibleMods.filter(
		({ info }) => !info.installedHash,
	);

	return (
		<>
			<Stack gap="lg">
				{mods.compatibleMods.length > 0 && (
					<>
						{!game.exePath && (
							<Alert color="orange">{t("gameNotInstalledWarning")}</Alert>
						)}
						{installedMods.length > 0 && (
							<Stack>
								<Divider label={t("installedMods")} />
								<GameModsTable
									game={game}
									mods={installedMods}
									remoteConfigs={remoteConfigs}
									highlightOnHover
								/>
								{game.exePath && (
									<Box px="xs">
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
									</Box>
								)}
							</Stack>
						)}
						{notInstalledMods.length > 0 && (
							<Stack>
								<Divider label={t("availableMods")} />
								<GameModsTable
									game={game}
									mods={notInstalledMods}
									remoteConfigs={remoteConfigs}
									highlightOnHover
								/>
							</Stack>
						)}
						{mods.hiddenMods.length > 0 && (
							<Stack>
								<Divider label={t("otherThings")} />
								<MutedText>{t("otherThingsDescription")}</MutedText>
								<GameModsTable
									game={game}
									mods={mods.hiddenMods}
									remoteConfigs={remoteConfigs}
									highlightOnHover
								/>
							</Stack>
						)}
					</>
				)}
			</Stack>
			{mods.incompatibleMods.length > 0 && (
				<Stack>
					<Divider label={t("incompatibleGameModsLabel")} />
					<MutedText>{t("incompatibleGameModsDescription")}</MutedText>
					<GameModsTable
						game={game}
						mods={mods.incompatibleMods}
						remoteConfigs={remoteConfigs}
						incompatible
					/>
				</Stack>
			)}
		</>
	);
}
