import {
	Alert,
	Button,
	Code,
	CopyButton,
	Flex,
	Modal,
	Stack,
} from "@mantine/core";
import { useModLoaders } from "@hooks/use-backend-data";
import {
	Game,
	installMod,
	openGameFolder,
	openGameModsFolder,
	startGame,
	uninstallMod,
} from "@api/bindings";
import { Fragment, useMemo, useState } from "react";
import { GameName } from "./game-name";
import { CommandButton } from "@components/command-button";
import {
	IconCopy,
	IconDownload,
	IconFolder,
	IconFolderCog,
	IconPlayerPlay,
	IconTrash,
} from "@tabler/icons-react";

type Props = {
	readonly game: Game;
	readonly onClose: () => void;
	readonly refreshGame: (gameId: string) => void;
};

export function InstalledGameModal(props: Props) {
	const [modLoaders] = useModLoaders();
	const [error, setError] = useState("");

	const handleError = (error: unknown) => {
		setError(`${error}`);
	};

	const debugData = useMemo(
		() => JSON.stringify(props.game, null, 2),
		[props.game],
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
				{error ? (
					<Alert
						color="red"
						style={{ overflow: "auto", flex: 1 }}
					>
						<pre>{error}</pre>
					</Alert>
				) : null}
				<Button.Group orientation="vertical">
					<CommandButton
						icon={<IconPlayerPlay />}
						onClick={() => startGame(props.game.id)}
					>
						Start Game
					</CommandButton>
					<CommandButton
						icon={<IconFolder />}
						onClick={() => openGameFolder(props.game.id)}
					>
						Open Game Folder
					</CommandButton>
					<CommandButton
						icon={<IconFolderCog />}
						onClick={() => openGameModsFolder(props.game.id).catch(handleError)}
					>
						Open Mods Folder
					</CommandButton>
				</Button.Group>
				{Object.values(modLoaders).map(
					(modLoader) =>
						// TODO need to filter these mods before checking the length.
						// Because we only show mods compatible with this game.
						modLoader.mods.length > 0 && (
							<Fragment key={modLoader.id}>
								<label>{modLoader.id} mods</label>
								<Button.Group orientation="vertical">
									{modLoader.mods
										.filter(
											(mod) =>
												mod.scriptingBackend === props.game.scriptingBackend,
										)
										.map((mod) =>
											props.game.installedMods.includes(mod.id) ? (
												<CommandButton
													icon={<IconTrash />}
													key={mod.name}
													onClick={async () => {
														await uninstallMod(props.game.id, mod.id);
														props.refreshGame(props.game.id);
													}}
												>
													Uninstall {mod.name}
												</CommandButton>
											) : (
												<CommandButton
													icon={<IconDownload />}
													key={mod.name}
													onClick={async () => {
														await installMod(
															modLoader.id,
															mod.id,
															props.game.id,
														);
														props.refreshGame(props.game.id);
													}}
												>
													Install {mod.name}
												</CommandButton>
											),
										)}
								</Button.Group>
							</Fragment>
						),
				)}
				<Stack gap="xs">
					<Flex
						align="end"
						justify="space-between"
					>
						<label>Debug Data</label>
						<CopyButton value={debugData}>
							{({ copied, copy }) => (
								<Button
									color="green"
									leftSection={<IconCopy />}
									onClick={copy}
									size="xs"
									variant={copied ? "filled" : "default"}
								>
									Copy
								</Button>
							)}
						</CopyButton>
					</Flex>
					<Code block>{debugData}</Code>
				</Stack>
			</Stack>
		</Modal>
	);
}
