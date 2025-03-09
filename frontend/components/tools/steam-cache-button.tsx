import { commands } from "@api/bindings";
import { CommandButton } from "@components/command-button";
import { useLocalization } from "@hooks/use-localization";
import { Button, Flex, Modal, Stack } from "@mantine/core";
import { IconHammer } from "@tabler/icons-react";
import { useCallback, useState } from "react";

export function SteamCacheButton() {
	const t = useLocalization("steamCache");
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
				{t("resetSteamCacheButton")}
			</Button>
			<Modal
				centered
				opened={isModalOpen}
				onClose={handleClose}
				title={t("resetSteamCacheModalTitle")}
			>
				<Stack>
					<span>{t("resetSteamCacheDescription")}</span>
					<Flex justify="center">
						<CommandButton
							onClick={commands.resetSteamCache}
							onSuccess={() => setShowSteamRestartPrompt(true)}
							leftSection={<IconHammer />}
						>
							{t("resetSteamCacheButton")}
						</CommandButton>
					</Flex>
					{showSteamRestartPrompt && <span>{t("resetSteamCacheSuccess")}</span>}
				</Stack>
			</Modal>
		</>
	);
}
