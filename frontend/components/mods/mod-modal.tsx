import { Modal, Stack } from "@mantine/core";
import { downloadMod, openModFolder, openModLoaderFolder } from "@api/bindings";
import { CommandButton } from "@components/command-button";
import { IconDownload, IconFolderCog } from "@tabler/icons-react";
import { CommandButtonGroup } from "@components/command-button-group";
import { DebugData } from "@components/debug-data";
import { UnifiedMod } from "@hooks/use-unified-mods";

type Props = {
	readonly mod: UnifiedMod;
	readonly onClose: () => void;
};

export function ModModal(props: Props) {
	const isDownloadAvailable = Boolean(props.mod.remote?.latestVersion.url);

	return (
		<Modal
			centered
			onClose={props.onClose}
			opened
			size="lg"
			title={props.mod.remote?.title ?? props.mod.common.id}
		>
			<Stack>
				<CommandButtonGroup label="Mod Actions">
					{props.mod.local && (
						<CommandButton
							leftSection={<IconFolderCog />}
							onClick={() => openModFolder(props.mod.common.id)}
						>
							Open mod folder
						</CommandButton>
					)}
					{!isDownloadAvailable && !props.mod.local && (
						<CommandButton
							leftSection={<IconFolderCog />}
							onClick={() => openModLoaderFolder(props.mod.common.loaderId)}
						>
							Open mod loader folder
						</CommandButton>
					)}
					{isDownloadAvailable && (
						<CommandButton
							leftSection={<IconDownload />}
							onClick={() => downloadMod(props.mod.common.id)}
						>
							Download mod
						</CommandButton>
					)}
				</CommandButtonGroup>
				<DebugData data={props.mod} />
			</Stack>
		</Modal>
	);
}
