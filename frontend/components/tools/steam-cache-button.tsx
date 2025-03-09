import { commands } from "@api/bindings";
import { CommandButton } from "@components/command-button";
import { useLocalization } from "@hooks/use-localization";
import { Flex, Menu, Modal, Stack } from "@mantine/core";
import { IconBrandSteam, IconHammer } from "@tabler/icons-react";
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
			<Menu.Item
				onClick={handleOpen}
				leftSection={<IconBrandSteam />}
			>
				{t("resetSteamCacheButtonOpenModal")}
			</Menu.Item>
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
							confirmationText={t("resetSteamCacheDescription")}
						>
							{t("resetSteamCacheButtonOpenModal")}
						</CommandButton>
					</Flex>
					{showSteamRestartPrompt && <span>{t("resetSteamCacheSuccess")}</span>}
				</Stack>
			</Modal>
		</>
	);
}
