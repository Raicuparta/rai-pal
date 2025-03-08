import { Button, Group, Modal, Stack, Text } from "@mantine/core";
import { IconAppWindowFilled, IconPlaylistAdd } from "@tabler/icons-react";
import { useCallback, useEffect, useState } from "react";
import { commands } from "@api/bindings";
import { open as openDialog } from "@tauri-apps/plugin-dialog";
import { useAtomValue } from "jotai";
import { loadingTasksAtom } from "@hooks/use-data";
import { useAsyncCommand } from "@hooks/use-async-command";
import { useLocalization } from "@hooks/use-localization";

export function AddGame() {
	const t = useLocalization("addGame");
	const [isOpen, setIsOpen] = useState(false);
	const isLoading = useAtomValue(loadingTasksAtom);

	const [executeAddGame] = useAsyncCommand(commands.addGame);

	const handleClick = useCallback(async () => {
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
	}, [executeAddGame]);

	useEffect(() => {
		if (isLoading) setIsOpen(false);
	}, [isLoading]);

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
						h="20em"
						onClick={handleClick}
					>
						<Group>
							<IconAppWindowFilled fontSize={50} />
							<Text>{t("dropField")}</Text>
						</Group>
					</Button>
					<Text>{t("note")}</Text>
				</Stack>
			</Modal>
		</>
	);
}
