import {
	ActionIcon,
	Button,
	Code,
	Group,
	Modal,
	Stack,
	Text,
} from "@mantine/core";
import {
	IconAppWindowFilled,
	IconDots,
	IconFolderFilled,
	IconPlaylistAdd,
	IconTrash,
} from "@tabler/icons-react";
import { useEffect, useState } from "react";
import { commands } from "@api/bindings";
import { open as openDialog } from "@tauri-apps/plugin-dialog";
import { useAsyncCommand } from "@hooks/use-async-command";
import { useLocalization } from "@hooks/use-localization";

export function AddGame() {
	const { t } = useLocalization("manualGames");
	const [isOpen, setIsOpen] = useState(false);
	const [directories, setDirectories] = useState<string[]>([]);

	const [executeAddGame] = useAsyncCommand(commands.addGame);
	const [executeAddGameDirectory] = useAsyncCommand(commands.addGameDirectory);

	const loadDirectories = () =>
		commands.getManualGameDirectories().then(setDirectories);

	useEffect(() => {
		if (isOpen) {
			loadDirectories();
		}
	}, [isOpen]);

	const handleFileClick = async () => {
		const path = await openDialog({
			multiple: false,
			title: "Select the game executable",
			filters: [
				{
					extensions: ["exe"],
					name: "Windows executable",
				},
				{
					extensions: ["*"],
					name: "Other executable",
				},
			],
		});
		if (!path) return;

		await executeAddGame(path).then(() => setIsOpen(false));
	};

	const handleDirectoryClick = async () => {
		const path = await openDialog({
			multiple: false,
			title: "Select a game directory",
			directory: true,
		});
		if (!path) return;

		await executeAddGameDirectory(path).then(() => setIsOpen(false));
	};

	const handleRemoveDirectory = async (path: string) => {
		await commands.removeGameDirectory(path);
		await commands.refreshGames("Manual");
		await loadDirectories();
	};

	return (
		<>
			<Button
				onClick={() => setIsOpen(true)}
				leftSection={<IconPlaylistAdd />}
			>
				{t("button")}
			</Button>
			<Modal
				opened={isOpen}
				centered
				size="lg"
				onClose={() => setIsOpen(false)}
				title={t("title")}
			>
				<Stack>
					{directories.length > 0 && (
						<Stack>
							<Text>{t("savedDirectories")}</Text>
							{directories.map((dir) => (
								<Group key={dir}>
									<Code style={{ flex: 1 }}>{dir}</Code>
									<ActionIcon
										color="red"
										variant="subtle"
										onClick={() => handleRemoveDirectory(dir)}
									>
										<IconTrash />
									</ActionIcon>
								</Group>
							))}
						</Stack>
					)}
					<Button
						size="lg"
						leftSection={<IconAppWindowFilled />}
						onClick={handleFileClick}
						rightSection={<IconDots />}
					>
						{t("selectGameExecutable")}
					</Button>
					<Button
						size="lg"
						leftSection={<IconFolderFilled />}
						onClick={handleDirectoryClick}
						rightSection={<IconDots />}
					>
						{t("selectGamesDirectory")}
					</Button>
					<Text>{t("fileDropNote")}</Text>
				</Stack>
			</Modal>
		</>
	);
}
