import { commands } from "@api/bindings";
import { CommandButton } from "@components/command-button";
import { useLocalization } from "@hooks/use-localization";
import { Flex, Modal, Stack } from "@mantine/core";
import { IconSquareRoundedPlus } from "@tabler/icons-react";
import { useState } from "react";

type Props = {
	readonly isOpen: boolean;
	readonly onClose: () => void;
};

export function SteamShortcutModal(props: Props) {
	const { t } = useLocalization("steamShortcut");
	const [showSuccess, setShowSuccess] = useState(false);

	return (
		<Modal
			centered
			opened={props.isOpen}
			onClose={() => {
				props.onClose();
				setShowSuccess(false);
			}}
			title={t("addRaiPalSteamShortcutModalTitle")}
		>
			<Stack>
				<span>{t("addRaiPalSteamShortcutDescription")}</span>
				<Flex justify="center">
					<CommandButton
						onClick={commands.addRaiPalSteamShortcut}
						onSuccess={() => setShowSuccess(true)}
						leftSection={<IconSquareRoundedPlus />}
					>
						{t("addRaiPalSteamShortcutButton")}
					</CommandButton>
				</Flex>
				{showSuccess && <span>{t("addRaiPalSteamShortcutSuccess")}</span>}
			</Stack>
		</Modal>
	);
}
