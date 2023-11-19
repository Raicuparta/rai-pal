import { Button, Flex, Modal, Stack, Text } from "@mantine/core";
import { IconAppWindowFilled, IconPlaylistAdd } from "@tabler/icons-react";
import { useCallback, useEffect, useState } from "react";
import styles from "./installed-games.module.css";
import { addGame } from "@api/bindings";
import { dialog } from "@tauri-apps/api";
import { useUpdateData } from "@hooks/use-update-data";
import { useAtomValue } from "jotai";
import { loadingAtom } from "@hooks/use-data";

export function AddGame() {
	const [isOpen, setIsOpen] = useState(false);
	const updateData = useUpdateData();
	const isLoading = useAtomValue(loadingAtom);

	const handleClick = useCallback(async () => {
		const result = await dialog.open({
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
		if (!result || Array.isArray(result)) return;

		await addGame(result);

		updateData();
	}, [updateData]);

	useEffect(() => {
		if (isLoading) setIsOpen(false);
	}, [isLoading]);

	return (
		<>
			<Button
				onClick={() => setIsOpen(true)}
				leftSection={<IconPlaylistAdd />}
			>
				Add game...
			</Button>
			<Modal
				opened={isOpen}
				centered
				size="lg"
				onClose={() => setIsOpen(false)}
				title="Add game"
				className={styles.modal}
			>
				<Stack>
					<Button
						fullWidth
						h="20em"
						onClick={handleClick}
					>
						<Flex
							gap="md"
							align="center"
						>
							<IconAppWindowFilled fontSize={50} />
							<Stack gap={0}>
								<Text>Drag and drop a game&apos;s executable file here</Text>
								<Text>or click to select a file</Text>
							</Stack>
						</Flex>
					</Button>
					<Text>
						Note: you can drop game executable files anywhere on Rai Pal&apos;s
						window to add them to the installed game list without opening this
						dialog.
					</Text>
				</Stack>
			</Modal>
		</>
	);
}
