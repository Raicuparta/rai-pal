import { Button, Container, Stack, Tooltip } from "@mantine/core";
import { resetLocalStorage } from "@util/local-storage";
import { IconFolderCode, IconRotateDot, IconTrash } from "@tabler/icons-react";
import { CommandButton } from "@components/command-button";
import { commands } from "@api/bindings";
import { SteamCacheButton } from "./steam-cache-button";
import { useGetTranslated } from "@hooks/use-translations";

export function ToolsPage() {
	const t = useGetTranslated("toolsPage");

	return (
		<Container size="xs">
			<Stack>
				<CommandButton
					onClick={commands.openLogsFolder}
					leftSection={<IconFolderCode />}
					justify="center"
				>
					{t("openLogsFolderButton")}
				</CommandButton>
				<Tooltip
					label={t("resetRaiPalSettingsTooltip")}
					position="bottom"
				>
					<Button
						onClick={resetLocalStorage}
						leftSection={<IconRotateDot />}
					>
						{t("resetRaiPalSettingsButton")}
					</Button>
				</Tooltip>
				<Tooltip
					label={t("clearCacheTooltip")}
					position="bottom"
				>
					<CommandButton
						onClick={commands.clearCache}
						leftSection={<IconTrash />}
					>
						{t("clearCacheButton")}
					</CommandButton>
				</Tooltip>
				<SteamCacheButton />
			</Stack>
		</Container>
	);
}
