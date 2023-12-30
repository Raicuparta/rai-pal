import {
	Button,
	Divider,
	Group,
	Modal,
	Stack,
	Table,
	Tooltip,
} from "@mantine/core";
import {
	ProviderId,
	openGameFolder,
	openGameModsFolder,
	refreshGame,
	removeGame,
	startGame,
	startGameExe,
} from "@api/bindings";
import { useMemo } from "react";
import { ItemName } from "../item-name";
import { CommandButton } from "@components/command-button";
import {
	Icon,
	IconAppWindow,
	IconBooks,
	IconBrandSteam,
	IconBrandXbox,
	IconBrowser,
	IconCircleLetterG,
	IconDeviceGamepad,
	IconFolder,
	IconFolderCog,
	IconFolderOpen,
	IconPlayerPlay,
	IconRefresh,
	IconSquareLetterE,
	IconTrash,
} from "@tabler/icons-react";
import { steamCommands } from "../../util/steam";
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

type Props = {
	readonly game: ProcessedInstalledGame;
	readonly onClose: () => void;
};

const providerIcons: Record<ProviderId, Icon> = {
	Manual: IconDeviceGamepad,
	Steam: IconBrandSteam,
	Epic: IconSquareLetterE,
	Gog: IconCircleLetterG,
	Xbox: IconBrandXbox,
};

function getProviderIcon(providerId: ProviderId) {
	return providerIcons[providerId] ?? IconDeviceGamepad;
}

export function InstalledGameModal(props: Props) {
	const modLoaderMap = useAtomValue(modLoadersAtom);
	const mods = useUnifiedMods();

	const filteredMods = useMemo(() => {
		return Object.values(mods).filter(
			(mod) => mod.common.id in props.game.installedModVersions,
		);
	}, [mods, props.game.installedModVersions]);

	const ProviderIcon = getProviderIcon(props.game.providerId);

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
							props.game.thumbnailUrl,
							props.game.providerId,
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
						{props.game.providerId === "Steam" && (
							<CommandDropdown>
								<CommandButton
									leftSection={<IconAppWindow />}
									onClick={() => startGameExe(props.game.id)}
								>
									Start Game Executable
								</CommandButton>
								{/* TODO: Implement this in a generic way on the Rust side, for every provider. */}
								<CommandButton
									leftSection={<ProviderIcon />}
									onClick={() => startGame(props.game.id)}
								>
									Start Game via {props.game.providerId}
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
					{/* TODO: Implement this in a generic way on the Rust side, for every provider. */}
					{props.game.providerId === "Steam" && (
						<CommandDropdown
							label={props.game.providerId}
							icon={<ProviderIcon />}
						>
							<CommandButton
								leftSection={<IconBrowser />}
								onClick={() =>
									steamCommands.openStorePage(props.game.steamLaunch?.appId)
								}
							>
								Open Store Page
							</CommandButton>
							<CommandButton
								leftSection={<IconBooks />}
								onClick={() =>
									steamCommands.showInLibrary(props.game.steamLaunch?.appId)
								}
							>
								Show in Library
							</CommandButton>
						</CommandDropdown>
					)}
					{props.game.providerId === "Manual" && (
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
				<DebugData data={props.game} />
			</Stack>
		</Modal>
	);
}
