import { Button, Flex, Modal, Stack, Text } from "@mantine/core";
import { IconAppWindowFilled, IconPlaylistAdd } from "@tabler/icons-react";
import { useState } from "react";
import styles from "./installed-games.module.css";

export function AddGame() {
	const [isOpen, setIsOpen] = useState(false);

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
					>
						<Flex
							gap="md"
							align="center"
						>
							<IconAppWindowFilled fontSize={50} />
							<Text
								size="xl"
								ta="left"
							>
								<div>Drag and drop a game&apos;s executable file here</div>
								<div>or click to select a file</div>
							</Text>
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
