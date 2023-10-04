import { deleteSteamAppinfoCache } from "@api/bindings";
import { CommandButton } from "@components/command-button";
import { Button, Modal, Stack } from "@mantine/core";
import { IconHammer } from "@tabler/icons-react";
import { useState } from "react";

export function FixOwnedGamesButton() {
	const [isModalOpen, setIsModalOpen] = useState(false);
	const [showSteamRestartPrompt, setShowSteamRestartPrompt] = useState(false);

	return (
		<>
			<Button
				onClick={() => setIsModalOpen(true)}
				leftSection={<IconHammer />}
			>
				Fix list
			</Button>
			<Modal
				centered
				opened={isModalOpen}
				onClose={() => setIsModalOpen(false)}
				title="Fix owned game list"
			>
				<Stack>
					<span>
						Use this if the list is showing games you don&apos;t actually own on
						Steam.
					</span>
					<span>
						This will delete Steam&apos;s cache file, and then you&apos;ll have
						to restart Steam. You won&apos;t lose any game files or saves or
						anything important.
					</span>
					<span>
						You&apos;ll get an error if the file doesn&apos;t exist, but
						that&apos;s fine.
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
							The cache file has been deleted. Please restart Steam now, and
							then refresh the owned game list.
						</Stack>
					)}
				</Stack>
			</Modal>
		</>
	);
}
