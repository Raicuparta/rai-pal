import { Modal, Stack } from "@mantine/core";
import {
	downloadMod,
	openModFolder,
	runRunnableWithoutGame,
} from "@api/bindings";
import { CommandButton } from "@components/command-button";
import {
	IconDownload,
	IconFolderCog,
	IconPlayerPlay,
	IconRefreshAlert,
} from "@tabler/icons-react";
import { DebugData } from "@components/debug-data";
import { UnifiedMod } from "@hooks/use-unified-mods";
import { ItemName } from "@components/item-name";
import { getModTitle } from "../../util/game-mod";

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
					{getModTitle(props.mod)}
				</ItemName>
			}
		>
			<Stack>
				{props.mod.local && props.mod.local.manifest?.runnable && (
					<CommandButton
						leftSection={<IconPlayerPlay />}
						onClick={() => runRunnableWithoutGame(props.mod.common.id)}
					>
						Run
					</CommandButton>
				)}
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
