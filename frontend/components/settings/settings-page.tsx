import { Button, Container, Stack, Tooltip } from "@mantine/core";
import { resetLocalStorage } from "../../util/local-storage";
import { IconFolderCode, IconRotateDot, IconTrash } from "@tabler/icons-react";
import { CommandButton } from "@components/command-button";
import { commands } from "@api/bindings";
import { clearDataCache } from "../../util/data-cache";

export function SettingsPage() {
	return (
		<Container size="xs">
			<Stack>
				<CommandButton
					onClick={commands.openLogsFolder}
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
				<Button
					onClick={clearDataCache}
					leftSection={<IconTrash />}
				>
					Clear cached games
				</Button>
			</Stack>
		</Container>
	);
}
