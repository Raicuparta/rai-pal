import { Alert, Box, Divider, Stack, Table, Text } from "@mantine/core";
import { DbGame, commands } from "@api/bindings";
import { ReactNode, useCallback } from "react";
import { CommandButton } from "@components/command-button";
import { IconTrash } from "@tabler/icons-react";
import { GameModRow } from "./game-mod-row";
import { useLocalization } from "@hooks/use-localization";
import { useCommandData } from "@hooks/use-command-data";
import { GameModsData, GameModsPart } from "@hooks/use-selected-game";

type Props = {
	readonly game: DbGame;
	readonly mods: GameModsData;
};

function GameModsSection({
	game,
	mods,
	remoteConfigModIds,
	incompatible,
	header,
	label,
	description,
}: {
	readonly game: DbGame;
	readonly mods: GameModsPart[];
	readonly remoteConfigModIds: ReadonlySet<string>;
	readonly label: string;
	readonly description?: string;
	readonly incompatible?: boolean;
	readonly header?: ReactNode;
}) {
	return (
		<Stack gap={0}>
			<Divider label={label} />
			{description && (
				<Text
					opacity={0.4}
					size="xs"
					ta="center"
					ml="xs"
				>
					{description}
				</Text>
			)}
			<Table>
				<Table.Tbody>
					{header}
					{mods.map(({ mod, info }) => (
						<GameModRow
							key={mod.id}
							game={game}
							mod={mod}
							info={info}
							hasRemoteConfig={remoteConfigModIds.has(mod.id)}
							incompatible={incompatible}
						/>
					))}
				</Table.Tbody>
			</Table>
		</Stack>
	);
}

export function GameMods({ game, mods }: Props) {
	const { t } = useLocalization("gameModal");
	const getRemoteConfigModIds = useCallback(
		() => commands.getRemoteConfigModIds(game.providerId, game.gameId),
		[game],
	);
	const [remoteConfigModIds] = useCommandData(getRemoteConfigModIds, []);
	const remoteConfigModIdSet = new Set(remoteConfigModIds);

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
								<GameModsSection
									label={t("installedMods")}
									game={game}
									mods={installedMods}
									remoteConfigModIds={remoteConfigModIdSet}
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
							<GameModsSection
								label={t("availableMods")}
								game={game}
								mods={notInstalledMods}
								remoteConfigModIds={remoteConfigModIdSet}
							/>
						)}
						{mods.hiddenMods.length > 0 && (
							<GameModsSection
								label={t("otherThings")}
								description={t("otherThingsDescription")}
								game={game}
								mods={mods.hiddenMods}
								remoteConfigModIds={remoteConfigModIdSet}
							/>
						)}
					</>
				)}
			</Stack>
			{mods.incompatibleMods.length > 0 && (
				<GameModsSection
					label={t("incompatibleGameModsLabel")}
					description={t("incompatibleGameModsDescription")}
					game={game}
					mods={mods.incompatibleMods}
					remoteConfigModIds={remoteConfigModIdSet}
					incompatible
				/>
			)}
		</>
	);
}
