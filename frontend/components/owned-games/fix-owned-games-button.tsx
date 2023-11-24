import { deleteSteamAppinfoCache } from "@api/bindings";
import { CommandButton } from "@components/command-button";
import { Button, Modal, Stack } from "@mantine/core";
import { IconHammer } from "@tabler/icons-react";
import { useCallback, useState } from "react";

export function FixOwnedGamesButton() {
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
				Fix list
			</Button>
			<Modal
				centered
				opened={isModalOpen}
				onClose={handleClose}
				title="Fix owned game list"
			>
				<Stack>
					<span>
						Use this if the list is showing games you don&apos;t actually own on
						Steam.
					</span>
					<span>
						This will reset Steam&apos;s cache, and then you&apos;ll have to
						restart Steam.
					</span>
					<span>
						You&apos;ll get an error if the file has already been deleted.
					</span>
					<CommandButton
						onClick={() => deleteSteamAppinfoCache()}
						onSuccess={() => setShowSteamRestartPrompt(true)}
						leftSection={<IconHammer />}
					>
						Delete Steam&apos;s Game Info Cache
					</CommandButton>
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
