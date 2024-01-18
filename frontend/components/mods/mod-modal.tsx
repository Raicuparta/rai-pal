import { Modal, Stack } from "@mantine/core";
import { downloadMod, openModFolder } from "@api/bindings";
import { CommandButton } from "@components/command-button";
import {
	IconDownload,
	IconFolderCog,
	IconRefreshAlert,
} from "@tabler/icons-react";
import { DebugData } from "@components/debug-data";
import { UnifiedMod } from "@hooks/use-unified-mods";
import { ItemName } from "@components/item-name";

type Props = {
	readonly mod: UnifiedMod;
	readonly onClose: () => void;
};

export function ModModal(props: Props) {
	const isDownloadAvailable = Boolean(props.mod.remote?.latestVersion?.url);
	const localVersion = props.mod.local?.manifest?.version;
	const remoteVersion = props.mod.remote?.latestVersion?.id;
	const isOutdated =
		localVersion && remoteVersion && remoteVersion !== localVersion;

	return (
		<Modal
			centered
			onClose={props.onClose}
			opened
			size="xl"
			title={
				<ItemName label={`by ${props.mod.remote?.author}`}>
					{props.mod.remote?.title ?? props.mod.common.id}
				</ItemName>
			}
		>
			<Stack>
				{props.mod.local && (
					<CommandButton
						leftSection={<IconFolderCog />}
						onClick={() => openModFolder(props.mod.common.id)}
					>
						Open mod folder
					</CommandButton>
				)}
				{isDownloadAvailable && (
					<CommandButton
						leftSection={isOutdated ? <IconRefreshAlert /> : <IconDownload />}
						onClick={() => downloadMod(props.mod.common.id)}
					>
						{isOutdated ? "Update mod" : "Download mod"}
					</CommandButton>
				)}
				<DebugData data={props.mod} />
			</Stack>
		</Modal>
	);
}
