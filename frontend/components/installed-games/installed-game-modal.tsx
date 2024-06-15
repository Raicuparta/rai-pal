import {
	Alert,
	Button,
	Divider,
	Group,
	Modal,
	Stack,
	Table,
	Tooltip,
} from "@mantine/core";
import {
	EngineVersion,
	EngineVersionRange,
	openGameFolder,
	openGameModsFolder,
	refreshGame,
	removeGame,
	startGame,
	startGameExe,
	uninstallAllMods,
} from "@api/bindings";
import { useMemo } from "react";
import { ItemName } from "../item-name";
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
import { ModalImage } from "@components/modal-image";
import { useAtomValue } from "jotai";
import { modLoadersAtom } from "@hooks/use-data";
import { DebugData } from "@components/debug-data";
import { useUnifiedMods } from "@hooks/use-unified-mods";
import { installedGamesColumns } from "./installed-games-columns";
import { TableItemDetails } from "@components/table/table-item-details";
import { ProcessedInstalledGame } from "@hooks/use-processed-installed-games";
import { GameModRow } from "./game-mod-row";
import { TableContainer } from "@components/table/table-container";
import { CommandDropdown } from "@components/command-dropdown";
import { getThumbnailWithFallback } from "../../util/fallback-thumbnail";
import { ProviderCommandButtons } from "@components/providers/provider-command-dropdown";
import { ProviderIcon } from "@components/providers/provider-icon";

type Props = {
	readonly game: ProcessedInstalledGame;
	readonly onClose: () => void;
};

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

export function InstalledGameModal(props: Props) {
	const modLoaderMap = useAtomValue(modLoadersAtom);
	const mods = useUnifiedMods();

	const filteredMods = useMemo(() => {
		return Object.values(mods).filter(
			(mod) =>
				(!mod.common.engine ||
					mod.common.engine === props.game.executable.engine?.brand) &&
				(!mod.common.unityBackend ||
					mod.common.unityBackend === props.game.executable.scriptingBackend) &&
				isVersionWithinRange(
					props.game.executable.engine?.version,
					mod.common.engineVersionRange,
				) &&
				!(
					mod.remote?.deprecated &&
					!props.game.installedModVersions[mod.common.id]
				),
		);
	}, [
		mods,
		props.game.executable.engine?.brand,
		props.game.executable.engine?.version,
		props.game.executable.scriptingBackend,
		props.game.installedModVersions,
	]);

	return (
		<Modal
			centered
			onClose={props.onClose}
			opened
			size="xl"
			title={
				<Group>
					<ModalImage
						src={getThumbnailWithFallback(
							props.game.thumbnailUrl ?? props.game.ownedGame?.thumbnailUrl,
							props.game.provider,
						)}
					/>
					<ItemName label={props.game.discriminator}>
						{props.game.name}
					</ItemName>
					<Tooltip label="Refresh game info">
						<CommandButton onClick={() => refreshGame(props.game.id)}>
							<IconRefresh />
						</CommandButton>
					</Tooltip>
				</Group>
			}
		>
			<Stack>
				<TableItemDetails
					columns={installedGamesColumns}
					item={props.game}
				/>
				<Group>
					<Button.Group>
						<CommandButton
							leftSection={<IconPlayerPlay />}
							onClick={() => startGame(props.game.id)}
						>
							Start Game
						</CommandButton>
						{props.game.startCommand && (
							<CommandDropdown>
								<CommandButton
									leftSection={<IconAppWindow />}
									onClick={() => startGameExe(props.game.id)}
								>
									Start Game Executable
								</CommandButton>
								<CommandButton
									leftSection={
										<ProviderIcon providerId={props.game.provider} />
									}
									onClick={() => startGame(props.game.id)}
								>
									Start Game via {props.game.provider}
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
							onClick={() => openGameFolder(props.game.id)}
						>
							Open Game Files Folder
						</CommandButton>
						<CommandButton
							leftSection={<IconFolderCog />}
							onClick={() => openGameModsFolder(props.game.id)}
						>
							Open Installed Mods Folder
						</CommandButton>
					</CommandDropdown>
					{props.game.ownedGame && (
						<ProviderCommandButtons
							game={props.game.ownedGame}
							isInstalled={true}
						/>
					)}
					{props.game.provider === "Manual" && (
						<CommandButton
							onClick={() => removeGame(props.game.id)}
							confirmationText="Are you sure you want to remove this game from Rai Pal?"
							onSuccess={props.onClose}
							leftSection={<IconTrash />}
						>
							Remove from Rai Pal
						</CommandButton>
					)}
				</Group>
				{(!props.game.executable.architecture ||
					!props.game.executable.operatingSystem) && (
					<Alert color="red">
						Failed to read some important information about this game. This
						could be due to the executable being protected. Some mods might fail
						to install.
					</Alert>
				)}
				{!props.game.executable.engine && (
					<Alert color="red">
						Failed to determine the engine for this game. Some mods might fail
						to install.
					</Alert>
				)}
				<Divider label="Mods" />
				<TableContainer bg="dark">
					<Table>
						<Table.Tbody>
							{filteredMods.map((mod) => (
								<GameModRow
									key={mod.common.id}
									game={props.game}
									mod={mod}
									modLoader={modLoaderMap[mod.common.loaderId]}
								/>
							))}
						</Table.Tbody>
					</Table>
				</TableContainer>
				<CommandButton
					confirmationText="You sure? This will delete all files in this game's mods folder. It won't delete any files from the actual game though."
					onClick={() => uninstallAllMods(props.game.id)}
					color="red"
					variant="light"
					leftSection={<IconTrash />}
				>
					Uninstall all mods
				</CommandButton>
				<DebugData data={props.game} />
			</Stack>
		</Modal>
	);
}
