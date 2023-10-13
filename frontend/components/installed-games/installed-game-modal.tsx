import { Button, Divider, Flex, Modal, Stack } from "@mantine/core";
import {
	Game,
	installMod,
	openGameFolder,
	openGameModsFolder,
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
import { useAppStore } from "@hooks/use-app-state";

type Props = {
	readonly game: Game;
	readonly onClose: () => void;
};

export function InstalledGameModal(props: Props) {
	const modLoaderMap = useAppStore((store) => store.data.modLoaders);

	const debugData = useMemo(
		() => JSON.stringify(props.game, null, 2),
		[props.game],
	);

	const modLoaders = useMemo(
		() =>
			Object.values(modLoaderMap ?? {}).map((modLoader) => ({
				...modLoader,
				mods: modLoader.mods.filter(
					(mod) => mod.id in props.game.availableMods,
				),
			})),
		[modLoaderMap, props.game.availableMods],
	);

	return (
		<Modal
			centered
			onClose={props.onClose}
			opened
			size="lg"
			title={
				<Flex>
					<GameName game={props.game} />
				</Flex>
			}
		>
			<Stack>
				<ModalImage src={props.game.thumbnailUrl} />
				<Flex
					justify="space-between"
					wrap="wrap"
					gap="md"
					w="100%"
				>
					<Stack gap="xs">
						<Divider label="Game actions" />
						<Button.Group
							orientation="vertical"
							w="fit-content"
						>
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
						</Button.Group>
					</Stack>
					{modLoaders.map(
						(modLoader) =>
							modLoader.mods.length > 0 && (
								<Stack
									key={modLoader.id}
									gap="xs"
								>
									<Divider label={modLoader.id.toUpperCase()} />
									<Button.Group
										orientation="vertical"
										w="fit-content"
									>
										{modLoader.mods.map((mod) =>
											props.game.availableMods[mod.id] ? (
												<CommandButton
													leftSection={<IconTrash />}
													key={mod.name}
													onClick={() => uninstallMod(props.game.id, mod.id)}
												>
													Uninstall {mod.name}
												</CommandButton>
											) : (
												<CommandButton
													leftSection={<IconTool />}
													key={mod.name}
													onClick={() =>
														installMod(modLoader.id, mod.id, props.game.id)
													}
												>
													{mod.kind === "Installable" ? "Install" : "Run"}{" "}
													{mod.name}
												</CommandButton>
											),
										)}
									</Button.Group>
								</Stack>
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
