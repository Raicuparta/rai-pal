import { commands } from "@api/bindings";
import { CommandButton } from "@components/command-button";
import { useLocalization } from "@hooks/use-localization";
import { Flex, Menu, Modal, Stack } from "@mantine/core";
import { IconCat, IconDots, IconHammer } from "@tabler/icons-react";
import { useState } from "react";

export function WineBepInExEnvironmentButton() {
	const t = useLocalization("wineBepInExEnvironment");
	const [isModalOpen, setIsModalOpen] = useState(false);
	const [showSteamRestartPrompt, setShowSteamRestartPrompt] = useState(false);

	return (
		<>
			<Menu.Item
				onClick={() => setIsModalOpen(true)}
				leftSection={<IconCat />}
				rightSection={<IconDots />}
			>
				{t("setUpEnvironmentButton")}
			</Menu.Item>
			<Modal
				centered
				opened={isModalOpen}
				onClose={() => {
					setIsModalOpen(false);
					setShowSteamRestartPrompt(false);
				}}
				title={t("setUpEnvironmentTitle")}
			>
				<Stack>
					<span>{t("setUpEnvironmentDescription")}</span>
					<Flex justify="center">
						<CommandButton
							onClick={commands.setUpWineBepinexEnvironment}
							onSuccess={() => setShowSteamRestartPrompt(true)}
							leftSection={<IconHammer />}
						>
							{t("setUpEnvironmentButton")}
						</CommandButton>
					</Flex>
					{showSteamRestartPrompt && (
						<span>{t("setUpEnvironmentSuccess")}</span>
					)}
				</Stack>
			</Modal>
		</>
	);
}
