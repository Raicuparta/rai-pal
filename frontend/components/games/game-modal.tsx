import {
	ActionIcon,
	Alert,
	Button,
	CloseIcon,
	Divider,
	Group,
	Modal,
	Stack,
	Table,
} from "@mantine/core";
import {
	EngineVersion,
	EngineVersionRange,
	Game,
	commands,
} from "@api/bindings";
import { useMemo } from "react";
import { CommandButton } from "@components/command-button";
import {
	IconAppWindow,
	IconFolder,
	IconFolderCog,
	IconFolderOpen,
	IconPlayerPlay,
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
import { ProviderIcon } from "@components/providers/provider-icon";
import { selectedGameAtom } from "./games-state";
import { ProviderCommandButtons } from "@components/providers/provider-command-dropdown";
import { GameRowInner } from "./game-row";
import { TableHead } from "@components/table/table-head";
import { gamesColumns } from "./games-columns";

type Props = {
	readonly game: Game;
};

const {
	openGameFolder,
	openGameModsFolder,
	refreshGame,
	removeGame,
	startGame,
	startGameExe,
	uninstallAllMods,
} = commands;

function isVersionWithinRange(
	version: EngineVersion | null | undefined,
	range: EngineVersionRange | null,
) {
	if (!version || !range) return true;

	const { major, minor, patch } = version.numbers;
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
	const { installedGame } = game;
	const modLoaderMap = useAtomValue(modLoadersAtom);
	const mods = useUnifiedMods();
	const setSelectedGame = useSetAtom(selectedGameAtom);

	const close = () => setSelectedGame(null);

	const filteredMods = useMemo(() => {
		return installedGame
			? Object.values(mods).filter(
					(mod) =>
						(!mod.common.engine ||
							mod.common.engine === installedGame.executable.engine?.brand) &&
						(!mod.common.unityBackend ||
							mod.common.unityBackend ===
								installedGame.executable.scriptingBackend) &&
						isVersionWithinRange(
							installedGame.executable.engine?.version,
							mod.common.engineVersionRange,
						) &&
						!(
							mod.remote?.deprecated &&
							!installedGame.installedModVersions[mod.common.id]
						),
				)
			: [];
	}, [mods, installedGame]);

	return (
		<Modal
			centered
			onClose={close}
			withCloseButton={false}
			opened
			size="xl"
			styles={{
				title: {
					width: "100%",
				},
			}}
			title={
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
					<ActionIcon
						onClick={close}
						variant="default"
					>
						<CloseIcon />
					</ActionIcon>
				</Group>
			}
		>
			<Stack>
				<Group>
					{installedGame && (
						<>
							<CommandButton
								onClick={() =>
									refreshGame({
										gameId: game.uniqueId,
										providerId: game.providerId,
									})
								}
								leftSection={<IconRefresh />}
							>
								Refresh
							</CommandButton>
							<Button.Group>
								<CommandButton
									leftSection={<IconPlayerPlay />}
									onClick={() => startGame(installedGame)}
								>
									Start Game
								</CommandButton>
								{installedGame.startCommand && (
									<CommandDropdown>
										<CommandButton
											leftSection={<IconAppWindow />}
											onClick={() => startGameExe(installedGame)}
										>
											Start Game Executable
										</CommandButton>
										<CommandButton
											leftSection={
												<ProviderIcon providerId={game.providerId} />
											}
											onClick={() => startGame(installedGame)}
										>
											Start Game via {game.providerId}
										</CommandButton>
									</CommandDropdown>
								)}
							</Button.Group>
							<CommandDropdown
								label="Folders"
								icon={<IconFolderOpen />}
							>
								<CommandButton
									leftSection={<IconFolder />}
									onClick={() => openGameFolder(installedGame)}
								>
									Open Game Files Folder
								</CommandButton>
								<CommandButton
									leftSection={<IconFolderCog />}
									onClick={() => openGameModsFolder(installedGame)}
								>
									Open Installed Mods Folder
								</CommandButton>
							</CommandDropdown>
						</>
					)}
					<ProviderCommandButtons
						game={game}
						isInstalled={true}
					/>
					{game.providerId === "Manual" && installedGame && (
						<CommandButton
							onClick={() => removeGame(installedGame)}
							confirmationText="Are you sure you want to remove this game from Rai Pal?"
							onSuccess={close}
							leftSection={<IconTrash />}
						>
							Remove from Rai Pal
						</CommandButton>
					)}
				</Group>
				{installedGame && (
					<>
						{!installedGame.executable.architecture && (
							<Alert color="red">
								Failed to read some important information about this game. This
								could be due to the executable being protected. Some mods might
								fail to install.
							</Alert>
						)}
						{!installedGame.executable.engine && (
							<Alert color="red">
								Failed to determine the engine for this game. Some mods might
								fail to install.
							</Alert>
						)}
						<Divider label="Mods" />
						<TableContainer bg="dark">
							<Table>
								<Table.Tbody>
									{filteredMods.map((mod) => (
										<GameModRow
											key={mod.common.id}
											game={installedGame}
											mod={mod}
											modLoader={modLoaderMap[mod.common.loaderId]}
										/>
									))}
								</Table.Tbody>
							</Table>
						</TableContainer>
						<CommandButton
							confirmationText="You sure? This will delete all files in this game's mods folder. It won't delete any files from the actual game though."
							onClick={() => uninstallAllMods(installedGame)}
							color="red"
							variant="light"
							leftSection={<IconTrash />}
						>
							Uninstall all mods
						</CommandButton>
					</>
				)}
				<DebugData data={game} />
			</Stack>
		</Modal>
	);
}
