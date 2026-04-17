import { commands } from "@api/bindings";
import { CommandButton } from "@components/command-button";
import { useLocalization } from "@hooks/use-localization";
import { Flex, Modal, Stack } from "@mantine/core";
import { IconHammer } from "@tabler/icons-react";
import { useState } from "react";

type Props = {
	isOpen: boolean;
	onClose: () => void;
};

export function SteamCacheModal(props: Props) {
	const t = useLocalization("steamCache");
	const [showSteamRestartPrompt, setShowSteamRestartPrompt] = useState(false);

	return (
		<Modal
			centered
			opened={props.isOpen}
			onClose={() => {
				props.onClose();
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
					>
						{t("resetSteamCacheButton")}
					</CommandButton>
				</Flex>
				{showSteamRestartPrompt && <span>{t("resetSteamCacheSuccess")}</span>}
			</Stack>
		</Modal>
	);
}
