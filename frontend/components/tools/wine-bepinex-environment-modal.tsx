import { commands } from "@api/bindings";
import { CommandButton } from "@components/command-button";
import { useLocalization } from "@hooks/use-localization";
import { Flex, Modal, Stack } from "@mantine/core";
import { useDisclosure } from "@mantine/hooks";
import { IconHammer } from "@tabler/icons-react";
import { useState } from "react";

type Props = {
	isOpen: boolean;
	onClose: () => void;
};

export function WineBepInExEnvironmentModal(props: Props) {
	const t = useLocalization("wineBepInExEnvironment");
	const [showSuccess, setShowSuccess] = useState(false);

	return (
		<Modal
			centered
			opened={props.isOpen}
			onClose={() => {
				props.onClose();
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
	);
}
