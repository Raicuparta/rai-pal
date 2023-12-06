import { Flex, Modal, Stack } from "@mantine/core";
import {
	InstalledGame,
	installMod,
	openGameFolder,
	openGameModsFolder,
	removeGame,
	startGame,
	uninstallMod,
} from "@api/bindings";
import { useMemo } from "react";
import { GameName } from "./game-name";
import { CommandButton } from "@components/command-button";
import {
	IconBooks,
	IconBrowser,
	IconFolder,
	IconFolderCog,
	IconPlayerPlay,
	IconTool,
	IconTrash,
} from "@tabler/icons-react";
import { CodeHighlight } from "@mantine/code-highlight";
import { steamCommands } from "../../util/steam";
import { ModalImage } from "@components/modal-image";
import { useAtomValue } from "jotai";
import { modLoadersAtom } from "@hooks/use-data";
import { CommandButtonGroup } from "@components/command-button-group";

type Props = {
	readonly game: InstalledGame;
	readonly onClose: () => void;
};

export function InstalledGameModal(props: Props) {
	const modLoaderMap = useAtomValue(modLoadersAtom);

	const debugData = useMemo(
		() => JSON.stringify(props.game, null, 2),
		[props.game],
	);

	const modLoaders = useMemo(
		() =>
			Object.values(modLoaderMap ?? {}).map((modLoader) => ({
				...modLoader,
				mods: Object.entries(modLoader.mods)
					.filter(([modId]) => modId in props.game.availableMods)
					.map(([, mod]) => mod),
			})),
		[modLoaderMap, props.game.availableMods],
	);

	return (
		<Modal
			centered
			onClose={props.onClose}
			opened
			size="lg"
			title={<GameName game={props.game} />}
		>
			<Stack>
				<ModalImage src={props.game.thumbnailUrl} />
				<Flex
					wrap="wrap"
					gap="md"
				>
					<CommandButtonGroup label="Game Actions">
						<CommandButton
							leftSection={<IconPlayerPlay />}
							onClick={() => startGame(props.game.id)}
						>
							Start Game
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
					</CommandButtonGroup>
					{modLoaders.map(
						(modLoader) =>
							modLoader.mods.length > 0 && (
								<CommandButtonGroup
									label={modLoader.id.toUpperCase()}
									key={modLoader.id}
								>
									{Object.entries(modLoader.mods).map(([modId, mod]) =>
										props.game.availableMods[modId] ? (
											<CommandButton
												leftSection={<IconTrash />}
												key={modId}
												onClick={() => uninstallMod(props.game.id, modId)}
											>
												Uninstall {mod.remoteMod?.title}
											</CommandButton>
										) : (
											<CommandButton
												leftSection={<IconTool />}
												key={modId}
												confirmationText="Attention: be careful when installing mods on multiplayer games! Anticheat can detect some mods and get you banned, even if the mods seem harmless."
												confirmationSkipId="install-mod-confirm"
												onClick={() =>
													installMod(modLoader.id, modId, props.game.id)
												}
											>
												{mod.localMod?.kind === "Installable"
													? "Install"
													: "Run"}{" "}
												{mod.remoteMod?.title ?? mod.localMod?.id}
												{/* TODO: handle local/remote mod stuff */}
												{/* <Text
													opacity={0.5}
													ml="xs"
													size="xs"
												>
													{props.game.executable.engine
														? ""
														: ` (${mod.engine}${
																mod.scriptingBackend
																	? ` ${mod.scriptingBackend}`
																	: ""
														  })`}
												</Text> */}
											</CommandButton>
										),
									)}
								</CommandButtonGroup>
							),
					)}
				</Flex>
				<Stack gap="xs">
					<label>Debug Data</label>
					<CodeHighlight
						code={debugData}
						language="json"
					/>
				</Stack>
			</Stack>
		</Modal>
	);
}
