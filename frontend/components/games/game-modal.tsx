import { Alert, Divider, Group, Modal, Stack, Table } from "@mantine/core";
import { EngineVersionRange, DbGame, commands } from "@api/bindings";
import { useMemo } from "react";
import { CommandButton } from "@components/command-button";
import {
	IconFolder,
	IconFolderCog,
	IconFolderOpen,
	IconRefresh,
	IconTrash,
} from "@tabler/icons-react";
import { useAtomValue, useSetAtom } from "jotai";
import { modLoadersAtom } from "@hooks/use-data";
import { DebugData } from "@components/debug-data";
import { useUnifiedMods } from "@hooks/use-unified-mods";
import { GameModRow } from "./game-mod-row";
import { TableContainer } from "@components/table/table-container";
import { CommandDropdown } from "@components/command-dropdown";
import { selectedGameAtom } from "./games-state";
import { ProviderCommandButtons } from "@components/providers/provider-command-dropdown";
import { GameRowInner } from "./game-row";
import { TableHead } from "@components/table/table-head";
import { gamesColumns } from "./games-columns";
import { useLocalization } from "@hooks/use-localization";
import { useCommandData } from "@hooks/use-command-data";
import { useAppEvent } from "@hooks/use-app-event";

type Props = {
	readonly game: DbGame;
};

function isVersionWithinRange(
	{
		engineVersionMajor: major,
		engineVersionMinor: minor,
		engineVersionPatch: patch,
	}: DbGame,
	range: EngineVersionRange | null,
) {
	if (!major || !range) return true;

	if (!major) return false;

	const { minimum, maximum } = range;

	if (minimum && minimum.major > major) return false;
	if (maximum && maximum.major < major) return false;
	if (
		minimum &&
		minimum.major === major &&
		minimum.minor != null &&
		minor != null &&
		minimum.minor > minor
	)
		return false;
	if (
		maximum &&
		maximum.major === major &&
		maximum.minor != null &&
		minor != null &&
		maximum.minor < minor
	)
		return false;
	if (
		minimum &&
		minimum.major === major &&
		minimum.minor === minor &&
		minimum.patch != null &&
		patch != null &&
		minimum.patch > patch
	)
		return false;
	if (
		maximum &&
		maximum.major === major &&
		maximum.minor === minor &&
		maximum.patch != null &&
		patch != null &&
		maximum.patch < patch
	)
		return false;

	return true;
}

export function GameModal({ game }: Props) {
	const t = useLocalization("gameModal");
	const modLoaderMap = useAtomValue(modLoadersAtom);
	const mods = useUnifiedMods();
	const setSelectedGame = useSetAtom(selectedGameAtom);
	const [installedModVersions, updateInstalledModVersions] = useCommandData(
		commands.getInstalledModVersions,
		() => (game.exePath ? { args: game } : { skip: true }),
		{},
	);

	useAppEvent("refreshGame", `installed-mods-${game.providerId}:${game.gameId}`, (foundId) => {
		if (
			foundId.providerId !== game.providerId ||
			foundId.gameId !== game.gameId
		)
			return;
		updateInstalledModVersions();
	});

	const close = () => setSelectedGame(null);

	const filteredMods = useMemo(() => {
		return Object.values(mods).filter(
			(mod) =>
				(!mod.common.engine || mod.common.engine === game.engineBrand) &&
				(!mod.common.unityBackend ||
					!game.unityBackend ||
					mod.common.unityBackend === game.unityBackend) &&
				isVersionWithinRange(game, mod.common.engineVersionRange) &&
				!(mod.remote?.deprecated && !installedModVersions[mod.common.id]),
		);
	}, [game, installedModVersions, mods]);

	return (
		<Modal
			centered
			onClose={close}
			opened
			size="xl"
			title={game.displayTitle}
		>
			<Stack>
				<Group align="start">
					<TableContainer>
						<Table>
							<Table.Thead>
								<TableHead columns={gamesColumns} />
							</Table.Thead>
							<Table.Tbody>
								<GameRowInner game={game} />
							</Table.Tbody>
						</Table>
					</TableContainer>
				</Group>
				<Group>
					<ProviderCommandButtons game={game} />
					{game.exePath && (
						<CommandDropdown
							label={t("foldersDropdown")}
							icon={<IconFolderOpen />}
						>
							<CommandButton
								leftSection={<IconFolder />}
								onClick={() => commands.openGameFolder(game)}
							>
								{t("openGameFilesFolder")}
							</CommandButton>
							<CommandButton
								leftSection={<IconFolderCog />}
								onClick={() => commands.openGameModsFolder(game)}
							>
								{t("openInstalledModsFolder")}
							</CommandButton>
						</CommandDropdown>
					)}
					{game.providerId === "Manual" && game.exePath && (
						<CommandButton
							onClick={() => commands.removeGame(game)}
							confirmationText={t("removeGameConfirmation")}
							onSuccess={close}
							leftSection={<IconTrash />}
						>
							{t("removeFromRaiPal")}
						</CommandButton>
					)}
					{game.exePath && (
						<CommandButton
							onClick={() => commands.refreshGame({ providerId: game.providerId, gameId: game.gameId })}
							leftSection={<IconRefresh />}
						>
							{t("refreshGame")}
						</CommandButton>
					)}
				</Group>
				{game.exePath && (
					<>
						{game.engineBrand && !game.architecture && (
							<Alert color="red">{t("failedToReadGameInfo")}</Alert>
						)}
						{!game.engineBrand && (
							<Alert color="red">{t("failedToDetermineEngine")}</Alert>
						)}
					</>
				)}
				{filteredMods.length > 0 && (
					<>
						<Divider label={t("gameModsLabel")} />
						{!game.exePath && (
							<Alert color="orange">{t("gameNotInstalledWarning")}</Alert>
						)}
						<TableContainer bg="dark">
							<Table>
								<Table.Tbody>
									{filteredMods.map((mod) => {
										const modLoader = modLoaderMap[mod.common.loaderId];

										return (
											modLoader && (
												<GameModRow
													key={mod.common.id}
													game={game}
													mod={mod}
													modLoader={modLoader}
													installedVersion={installedModVersions[mod.common.id]}
												/>
											)
										);
									})}
								</Table.Tbody>
							</Table>
						</TableContainer>
						{game.exePath && (
							<CommandButton
								confirmationText={t("uninstallAllModsConfirmation")}
								onClick={() => commands.uninstallAllMods(game)}
								color="red"
								variant="light"
								leftSection={<IconTrash />}
							>
								{t("uninstallAllModsButton")}
							</CommandButton>
						)}
					</>
				)}
				<DebugData data={game} />
			</Stack>
		</Modal>
	);
}
