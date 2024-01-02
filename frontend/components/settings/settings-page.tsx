import { Button, Container, Stack, Tooltip } from "@mantine/core";
import { resetLocalStorage } from "../../util/local-storage";
import { IconFolderCode, IconRotateDot } from "@tabler/icons-react";
import { CommandButton } from "@components/command-button";
import { openLogsFolder } from "@api/bindings";

export function SettingsPage() {
	return (
		<Container size="xs">
			<Stack>
				<CommandButton
					onClick={openLogsFolder}
					leftSection={<IconFolderCode />}
					justify="center"
				>
					Open Logs Folder
				</CommandButton>
				<Tooltip
					label="Will reset filters, confirmation dialogs, probably other stuff."
					position="bottom"
				>
					<Button
						onClick={resetLocalStorage}
						leftSection={<IconRotateDot />}
					>
						Reset settings to defaults
					</Button>
				</Tooltip>
			</Stack>
		</Container>
	);
}
