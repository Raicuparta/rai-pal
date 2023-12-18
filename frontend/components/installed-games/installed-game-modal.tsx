import {
	Box,
	Divider,
	Group,
	Modal,
	Stack,
	Table,
	Text,
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
	IconBrowser,
	IconDeviceGamepad,
	IconFolder,
	IconFolderCog,
	IconRefresh,
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
import { GameModButton } from "./game-mod-button";
import { TableContainer } from "@components/table/table-container";

type Props = {
	readonly game: ProcessedInstalledGame;
	readonly onClose: () => void;
};

const providerIcons: Record<ProviderId, Icon> = {
	Manual: IconDeviceGamepad,
	Steam: IconBrandSteam,
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
					<ModalImage src={props.game.thumbnailUrl} />
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
				<Divider label="Mods" />
				<TableContainer>
					<Table>
						<Table.Tbody>
							{filteredMods.map((mod) => (
								<Table.Tr key={mod.common.id}>
									<Table.Td ta="left">
										<ItemName label={`by ${mod.remote?.author}`}>
											{mod.remote?.title ?? mod.common.id}
										</ItemName>
										{mod.remote?.description && (
											<Text
												size="sm"
												opacity={0.5}
											>
												{mod.remote.description}
											</Text>
										)}
									</Table.Td>
									<Table.Td>
										<GameModButton
											key={mod.common.id}
											game={props.game}
											mod={mod}
											modLoader={modLoaderMap[mod.common.loaderId]}
										/>
									</Table.Td>
								</Table.Tr>
							))}
						</Table.Tbody>
					</Table>
				</TableContainer>
				<Box>
					<CommandButton
						leftSection={<IconFolderCog />}
						onClick={() => openGameModsFolder(props.game.id)}
					>
						Open Mods Folder
					</CommandButton>
				</Box>
				{props.game.providerId !== "Manual" && props.game.steamLaunch && (
					<>
						<Divider label={props.game.providerId} />
						<Group>
							<CommandButton
								leftSection={<ProviderIcon />}
								onClick={() => startGame(props.game.id)}
							>
								Start Game via {props.game.providerId}
							</CommandButton>
							<>
								<CommandButton
									leftSection={<IconBooks />}
									onClick={() =>
										steamCommands.showInLibrary(props.game.steamLaunch?.appId)
									}
								>
									Show in Steam Library
								</CommandButton>
								<CommandButton
									leftSection={<IconBrowser />}
									onClick={() =>
										steamCommands.openStorePage(props.game.steamLaunch?.appId)
									}
								>
									Open Steam Page
								</CommandButton>
							</>
						</Group>
					</>
				)}
				<Divider label="Game Files" />
				<TableItemDetails
					columns={installedGamesColumns}
					item={props.game}
				/>
				<Group>
					<CommandButton
						leftSection={<IconAppWindow />}
						onClick={() => startGameExe(props.game.id)}
					>
						Start Game Executable
					</CommandButton>
					<CommandButton
						leftSection={<IconFolder />}
						onClick={() => openGameFolder(props.game.id)}
					>
						Open Game Folder
					</CommandButton>
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
				<DebugData data={props.game} />
			</Stack>
		</Modal>
	);
}
