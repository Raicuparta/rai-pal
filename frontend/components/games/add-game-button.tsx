import { Button, Group, Modal, Stack, Text } from "@mantine/core";
import {
	IconAppWindowFilled,
	IconFolderFilled,
	IconPlaylistAdd,
} from "@tabler/icons-react";
import { useState } from "react";
import { commands } from "@api/bindings";
import { open as openDialog } from "@tauri-apps/plugin-dialog";
import { useAsyncCommand } from "@hooks/use-async-command";
import { useLocalization } from "@hooks/use-localization";

export function AddGame() {
	const { t } = useLocalization("addGame");
	const [isOpen, setIsOpen] = useState(false);

	const [executeAddGame] = useAsyncCommand(commands.addGame);
	const [executeAddGameDirectory] = useAsyncCommand(commands.addGameDirectory);

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
					<Button
						fullWidth
						h="10em"
						onClick={handleFileClick}
					>
						<Group>
							<IconAppWindowFilled fontSize={50} />
							<Text>{t("dropField")}</Text>
						</Group>
					</Button>
					<Button
						fullWidth
						h="10em"
						onClick={handleDirectoryClick}
					>
						<Group>
							<IconFolderFilled fontSize={50} />
							<Text>{t("directoryButton")}</Text>
						</Group>
					</Button>
					<Text>{t("note")}</Text>
				</Stack>
			</Modal>
		</>
	);
}
