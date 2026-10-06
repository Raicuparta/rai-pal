import { AvailableUpdate } from "@hooks/use-app-updater";
import { useLocalization } from "@hooks/use-localization";
import { Button, Group, Modal, ScrollArea, Stack, Text } from "@mantine/core";
import { IconChevronsUp, IconRefreshAlert, IconZzz } from "@tabler/icons-react";
import { useState } from "react";

type Props = {
	readonly update: AvailableUpdate | null;
	readonly onInstall: () => Promise<void>;
	readonly onIgnore: () => void;
};

export function AppUpdateModal(props: Props) {
	const { t } = useLocalization("appUpdate");
	const [isInstalling, setIsInstalling] = useState(false);

	const handleInstall = async () => {
		setIsInstalling(true);
		try {
			await props.onInstall();
		} finally {
			setIsInstalling(false);
		}
	};

	return (
		<Modal
			centered
			size="lg"
			opened={!!props.update}
			withCloseButton={false}
			onClose={props.onIgnore}
			closeOnClickOutside={false}
			closeOnEscape={false}
		>
			<Stack>
				<Group>
					<IconChevronsUp />
					<Text
						fz="xl"
						fw="bold"
					>
						{t("updateAvailableTitle", {
							version: props.update?.version ?? "",
						})}
					</Text>
				</Group>
				<ScrollArea.Autosize mah={400}>
					<Text style={{ whiteSpace: "pre-wrap" }}>
						{props.update?.body || t("noChangelog")}
					</Text>
				</ScrollArea.Autosize>
				<Group justify="space-around">
					<Button
						variant="default"
						onClick={props.onIgnore}
						disabled={isInstalling}
						leftSection={<IconZzz />}
					>
						{t("ignoreUpdate")}
					</Button>
					<Button
						variant="filled"
						onClick={handleInstall}
						loading={isInstalling}
						leftSection={<IconRefreshAlert />}
					>
						{t("updateNow")}
					</Button>
				</Group>
			</Stack>
		</Modal>
	);
}
