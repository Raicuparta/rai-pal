import { commands } from "@api/bindings";
import { CommandButton } from "@components/command-button";
import { useLocalization } from "@hooks/use-localization";
import { Flex, Menu, Modal, Stack } from "@mantine/core";
import { IconBrandSteam, IconDots, IconHammer } from "@tabler/icons-react";
import { useState } from "react";

export function SteamCacheButton() {
	const t = useLocalization("steamCache");
	const [isModalOpen, setIsModalOpen] = useState(false);
	const [showSteamRestartPrompt, setShowSteamRestartPrompt] = useState(false);

	return (
		<>
			<Menu.Item
				onClick={() => setIsModalOpen(true)}
				leftSection={<IconBrandSteam />}
				rightSection={<IconDots />}
			>
				{t("resetSteamCacheButton")}
			</Menu.Item>
			<Modal
				centered
				opened={isModalOpen}
				onClose={() => {
					setIsModalOpen(false);
					setShowSteamRestartPrompt(false);
				}}
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
							{t("resetSteamCacheButton")}
						</CommandButton>
					</Flex>
					{showSteamRestartPrompt && <span>{t("resetSteamCacheSuccess")}</span>}
				</Stack>
			</Modal>
		</>
	);
}
