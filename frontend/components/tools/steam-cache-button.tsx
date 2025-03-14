import { commands } from "@api/bindings";
import { CommandButton } from "@components/command-button";
import { Button, Flex, Modal, Stack } from "@mantine/core";
import { IconHammer } from "@tabler/icons-react";
import { useCallback, useState } from "react";

export function SteamCacheButton() {
	const [isModalOpen, setIsModalOpen] = useState(false);
	const [showSteamRestartPrompt, setShowSteamRestartPrompt] = useState(false);

	const handleOpen = useCallback(() => {
		setIsModalOpen(true);
	}, []);

	const handleClose = useCallback(() => {
		setIsModalOpen(false);
		setShowSteamRestartPrompt(false);
	}, []);

	return (
		<>
			<Button
				onClick={handleOpen}
				leftSection={<IconHammer />}
			>
				Reset Steam cache
			</Button>
			<Modal
				centered
				opened={isModalOpen}
				onClose={handleClose}
				title="Reset Steam cache"
			>
				<Stack>
					<span>
						Use this if Rai Pal is showing games you don&apos;t actually own on
						Steam.
					</span>
					<span>
						This will reset Steam&apos;s cache, and then you&apos;ll have to
						restart Steam.
					</span>
					<span>
						You&apos;ll get an error if the file has already been deleted.
					</span>
					<Flex justify="center">
						<CommandButton
							onClick={commands.resetSteamCache}
							onSuccess={() => setShowSteamRestartPrompt(true)}
							leftSection={<IconHammer />}
						>
							Reset Steam cache
						</CommandButton>
					</Flex>
					{showSteamRestartPrompt && (
						<Stack>
							The cache file has been deleted. Please restart Steam, wait a few
							seconds, and then press the refresh button on Rai Pal.
						</Stack>
					)}
				</Stack>
			</Modal>
		</>
	);
}