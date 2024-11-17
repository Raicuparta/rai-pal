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
	InstalledGame,
	ProviderId,
	commands,
} from "@api/bindings";
import { useEffect, useMemo, useState } from "react";
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
import { GameModRow } from "./game-mod-row";
import { TableContainer } from "@components/table/table-container";
import { CommandDropdown } from "@components/command-dropdown";
import {
	getFallbackThumbnail,
	getThumbnailWithFallback,
} from "@util/fallback-thumbnail";
import { ProviderCommandButtons } from "@components/providers/provider-command-dropdown";
import { ProviderIcon } from "@components/providers/provider-icon";

type Props = {
	readonly gameId: string;
	readonly providerId: ProviderId;
	readonly onClose: () => void;
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

export function InstalledGameModal(props: Props) {
	const modLoaderMap = useAtomValue(modLoadersAtom);
	const mods = useUnifiedMods();
	const [game, setGame] = useState<InstalledGame>();

	const filteredMods = useMemo(() => {
		if (!game) return [];

		return Object.values(mods).filter(
			(mod) =>
				(!mod.common.engine ||
					mod.common.engine === game.executable.engine?.brand) &&
				(!mod.common.unityBackend ||
					mod.common.unityBackend === game.executable.scriptingBackend) &&
				isVersionWithinRange(
					game.executable.engine?.version,
					mod.common.engineVersionRange,
				) &&
				!(mod.remote?.deprecated && !game.installedModVersions[mod.common.id]),
		);
	}, [mods, game]);

	useEffect(() => {
		commands.getInstalledGame(props.providerId, props.gameId).then((result) => {
			if (result.status === "ok") {
				// TODO: handle errors.
				setGame(result.data);
			}
		});
	}, [props.gameId, props.providerId]);

	if (!game) return null; // TODO show loader?

	return (
		<Modal
			centered
			onClose={props.onClose}
			opened
			size="xl"
			title={
				<Group>
					<ModalImage
						src={
							game
								? getThumbnailWithFallback(
										game.thumbnailUrl ?? game.ownedGame?.thumbnailUrl,
										game.provider,
									)
								: getFallbackThumbnail("Manual")
						}
					/>
					<ItemName label={game.discriminator}>{game.title.display}</ItemName>
					<Tooltip label="Refresh game info">
						<CommandButton onClick={() => refreshGame(game)}>
							<IconRefresh />
						</CommandButton>
					</Tooltip>
				</Group>
			}
		>
			<Stack>
				{game && (
					<>
						<TableItemDetails
							columns={installedGamesColumns}
							item={game}
						/>
						<Group>
							<Button.Group>
								<CommandButton
									leftSection={<IconPlayerPlay />}
									onClick={() => startGame(game)}
								>
									Start Game
								</CommandButton>
								{game.startCommand && (
									<CommandDropdown>
										<CommandButton
											leftSection={<IconAppWindow />}
											onClick={() => startGameExe(game)}
										>
											Start Game Executable
										</CommandButton>
										<CommandButton
											leftSection={<ProviderIcon providerId={game.provider} />}
											onClick={() => startGame(game)}
										>
											Start Game via {game.provider}
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
									onClick={() => openGameFolder(game)}
								>
									Open Game Files Folder
								</CommandButton>
								<CommandButton
									leftSection={<IconFolderCog />}
									onClick={() => openGameModsFolder(game)}
								>
									Open Installed Mods Folder
								</CommandButton>
							</CommandDropdown>
							{game.ownedGame && (
								<ProviderCommandButtons
									game={game.ownedGame}
									isInstalled={true}
								/>
							)}
							{game.provider === "Manual" && (
								<CommandButton
									onClick={() => removeGame(game)}
									confirmationText="Are you sure you want to remove this game from Rai Pal?"
									onSuccess={props.onClose}
									leftSection={<IconTrash />}
								>
									Remove from Rai Pal
								</CommandButton>
							)}
						</Group>
						{!game.executable.architecture && (
							<Alert color="red">
								Failed to read some important information about this game. This
								could be due to the executable being protected. Some mods might
								fail to install.
							</Alert>
						)}
						{!game.executable.engine && (
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
											game={game}
											mod={mod}
											modLoader={modLoaderMap[mod.common.loaderId]}
										/>
									))}
								</Table.Tbody>
							</Table>
						</TableContainer>
						<CommandButton
							confirmationText="You sure? This will delete all files in this game's mods folder. It won't delete any files from the actual game though."
							onClick={() => uninstallAllMods(game)}
							color="red"
							variant="light"
							leftSection={<IconTrash />}
						>
							Uninstall all mods
						</CommandButton>
						<DebugData data={game} />
					</>
				)}
			</Stack>
		</Modal>
	);
}
