import { Flex, Modal, Stack, Text, ThemeIcon, Tooltip } from "@mantine/core";
import {
	InstalledGame,
	installMod,
	openGameFolder,
	openGameModsFolder,
	removeGame,
	startGame,
	startGameExe,
	uninstallMod,
} from "@api/bindings";
import { useMemo } from "react";
import { GameName } from "./game-name";
import { CommandButton } from "@components/command-button";
import {
	IconAppWindow,
	IconBooks,
	IconBrowser,
	IconFolder,
	IconFolderCog,
	IconPlayerPlay,
	IconRefreshAlert,
	IconShoppingBag,
	IconTool,
	IconTrash,
} from "@tabler/icons-react";
import { steamCommands } from "../../util/steam";
import { ModalImage } from "@components/modal-image";
import { useAtomValue } from "jotai";
import { modLoadersAtom } from "@hooks/use-data";
import { CommandButtonGroup } from "@components/command-button-group";
import { DebugData } from "@components/debug-data";
import { useUnifiedMods } from "@hooks/use-unified-mods";
import { isOutdated } from "../../util/is-outdated";

type Props = {
	readonly game: InstalledGame;
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
			title={<GameName game={props.game} />}
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
					</CommandButtonGroup>
					{modLoaders.map(
						(modLoader) =>
							modLoader.mods.length > 0 && (
								<CommandButtonGroup
									label={modLoader.id.toUpperCase()}
									key={modLoader.id}
								>
									{/* TODO: these buttons could be extracted to a separate component, lots of stuff happening. */}
									{modLoader.mods.map((mod) => {
										const installedVersion =
											props.game.installedModVersions[mod.common.id];
										const outdated = isOutdated(
											installedVersion,
											mod.remote?.latestVersion.id,
										);

										return installedVersion ? (
											<Tooltip
												disabled={!outdated}
												label="Mod outdated. Reinstall it to update."
												key={mod.common.id}
											>
												<CommandButton
													leftSection={
														outdated ? (
															<ThemeIcon
																radius="xl"
																color="orange"
															>
																<IconRefreshAlert />
															</ThemeIcon>
														) : (
															<IconTrash />
														)
													}
													onClick={() =>
														uninstallMod(props.game.id, mod.common.id)
													}
												>
													Uninstall {mod.remote?.title ?? mod.common.id}{" "}
													<Text
														opacity={0.5}
														ml="xs"
														size="xs"
													>
														({installedVersion})
													</Text>
												</CommandButton>
											</Tooltip>
										) : (
											<CommandButton
												leftSection={<IconTool />}
												key={mod.common.id}
												confirmationText="Attention: be careful when installing mods on multiplayer games! Anticheat can detect some mods and get you banned, even if the mods seem harmless."
												confirmationSkipId="install-mod-confirm"
												onClick={() => installMod(mod.common.id, props.game.id)}
											>
												{modLoader.kind === "Installable" ? "Install" : "Run"}{" "}
												{mod.remote?.title ?? mod.common.id}
												{!props.game.executable.engine && (
													// TODO this text to separate component, it's used in multiple places.
													<Text
														opacity={0.5}
														ml="xs"
														size="xs"
													>
														({mod.common.engine})
													</Text>
												)}
												<Text
													opacity={0.5}
													ml="xs"
													size="xs"
												>
													({mod.remote?.latestVersion.id})
												</Text>
											</CommandButton>
										);
									})}
								</CommandButtonGroup>
							),
					)}
				</Flex>
				<DebugData data={props.game} />
			</Stack>
		</Modal>
	);
}
