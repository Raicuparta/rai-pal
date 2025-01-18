import { Button, Container, Stack, Tooltip } from "@mantine/core";
import { resetLocalStorage } from "@util/local-storage";
import { IconFolderCode, IconRotateDot, IconTrash } from "@tabler/icons-react";
import { CommandButton } from "@components/command-button";
import { commands } from "@api/bindings";
import { SteamCacheButton } from "./steam-cache-button";

export function ToolsPage() {
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
						Reset Rai Pal settings
					</Button>
				</Tooltip>
				<Tooltip
					label="Clears the game list cache used by Rai Pal."
					position="bottom"
				>
					<CommandButton
						onClick={commands.clearCache}
						leftSection={<IconTrash />}
					>
						Reset Rai Pal cache
					</CommandButton>
				</Tooltip>
				<SteamCacheButton />
			</Stack>
		</Container>
	);
}
