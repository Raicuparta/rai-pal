import { Flex, Modal, Stack } from "@mantine/core";
import {
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
	IconAppWindow,
	IconBooks,
	IconBrowser,
	IconFolder,
	IconFolderCog,
	IconPlayerPlay,
	IconRefresh,
	IconShoppingBag,
	IconTrash,
} from "@tabler/icons-react";
import { steamCommands } from "../../util/steam";
import { ModalImage } from "@components/modal-image";
import { useAtomValue } from "jotai";
import { modLoadersAtom } from "@hooks/use-data";
import { CommandButtonGroup } from "@components/command-button-group";
import { DebugData } from "@components/debug-data";
import { useUnifiedMods } from "@hooks/use-unified-mods";
import { installedGamesColumns } from "./installed-games-columns";
import { TableItemDetails } from "@components/table/table-item-details";
import { ProcessedInstalledGame } from "@hooks/use-processed-installed-games";
import { GameModButton } from "./game-mod-button";

type Props = {
	readonly game: ProcessedInstalledGame;
	readonly onClose: () => void;
};

export function InstalledGameModal(props: Props) {
	const modLoaderMap = useAtomValue(modLoadersAtom);
	const mods = useUnifiedMods();

	// TODO make less insane?
	const modLoaders = useMemo(
		() =>
			Object.values(modLoaderMap ?? {}).map((modLoader) => ({
				...modLoader,
				mods: Object.entries(mods)
					.filter(
						([modId, mod]) =>
							modId in props.game.installedModVersions &&
							mod.common.loaderId === modLoader.id,
					)
					.map(([, mod]) => mod),
			})),
		[modLoaderMap, mods, props.game.installedModVersions],
	);

	return (
		<Modal
			centered
			onClose={props.onClose}
			opened
			size="lg"
			title={
				<ItemName label={props.game.discriminator}>{props.game.name}</ItemName>
			}
		>
			<Stack>
				<ModalImage src={props.game.thumbnailUrl} />
				<Flex
					wrap="wrap"
					gap="md"
				>
					<CommandButtonGroup label="Game Actions">
						{props.game.providerId !== "Manual" && (
							<CommandButton
								leftSection={<IconPlayerPlay />}
								rightSection={<IconShoppingBag />}
								onClick={() => startGame(props.game.id)}
							>
								Start Game ({props.game.providerId})
							</CommandButton>
						)}
						<CommandButton
							leftSection={<IconPlayerPlay />}
							rightSection={<IconAppWindow />}
							onClick={() => startGameExe(props.game.id)}
						>
							Start Game (Exe)
						</CommandButton>
						<CommandButton
							leftSection={<IconFolder />}
							onClick={() => openGameFolder(props.game.id)}
						>
							Open Game Folder
						</CommandButton>
						<CommandButton
							leftSection={<IconFolderCog />}
							onClick={() => openGameModsFolder(props.game.id)}
						>
							Open Mods Folder
						</CommandButton>
						{props.game.steamLaunch && (
							<>
								<CommandButton
									leftSection={<IconBooks />}
									onClick={() =>
										steamCommands.showInLibrary(props.game.steamLaunch?.appId)
									}
								>
									Show in Library
								</CommandButton>
								<CommandButton
									leftSection={<IconBrowser />}
									onClick={() =>
										steamCommands.openStorePage(props.game.steamLaunch?.appId)
									}
								>
									Open Store Page
								</CommandButton>
							</>
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
						<CommandButton
							onClick={() => refreshGame(props.game.id)}
							leftSection={<IconRefresh />}
						>
							Refresh Game
						</CommandButton>
					</CommandButtonGroup>
					{modLoaders.map(
						(modLoader) =>
							modLoader.mods.length > 0 && (
								<CommandButtonGroup
									label={modLoader.id.toUpperCase()}
									key={modLoader.id}
								>
									{modLoader.mods.map((mod) => (
										<GameModButton
											key={mod.common.id}
											game={props.game}
											mod={mod}
											modLoader={modLoader}
										/>
									))}
								</CommandButtonGroup>
							),
					)}
				</Flex>
				<TableItemDetails
					columns={installedGamesColumns}
					item={props.game}
				/>
				<DebugData data={props.game} />
			</Stack>
		</Modal>
	);
}
