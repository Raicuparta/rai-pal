import { commands } from "@api/bindings";
import { CommandButton } from "@components/command-button";
import { useLocalization } from "@hooks/use-localization";
import { Button, Flex, Menu, Modal, Stack } from "@mantine/core";
import { useDisclosure } from "@mantine/hooks";
import { IconCat, IconDots, IconHammer } from "@tabler/icons-react";
import { useState } from "react";

export function WineBepInExEnvironmentButton() {
	const t = useLocalization("wineBepInExEnvironment");
	const [opened, { open, close }] = useDisclosure(false);
	const [showSuccess, setShowSuccess] = useState(false);

	return (
		<>
			<Button
				onClick={() => open()}
				leftSection={<IconCat />}
				rightSection={<IconDots />}
			>
				{t("setUpEnvironmentButton")}
			</Button>
			<Modal
				centered
				opened={opened}
				onClose={() => {
					close();
					setShowSuccess(false);
				}}
				title={t("setUpEnvironmentTitle")}
			>
				<Stack>
					<span>{t("setUpEnvironmentDescription")}</span>
					<Flex justify="center">
						<CommandButton
							onClick={commands.setUpWineBepinexEnvironment}
							onSuccess={() => setShowSuccess(true)}
							leftSection={<IconHammer />}
						>
							{t("setUpEnvironmentButton")}
						</CommandButton>
					</Flex>
					{showSuccess && <span>{t("setUpEnvironmentSuccess")}</span>}
				</Stack>
			</Modal>
		</>
	);
}
