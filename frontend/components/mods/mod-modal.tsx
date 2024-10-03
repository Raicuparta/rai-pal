import { Group, Modal, Stack } from "@mantine/core";
import { commands } from "@api/bindings";
import { CommandButton } from "@components/command-button";
import {
	IconDownload,
	IconFolderCog,
	IconPlayerPlay,
	IconRefreshAlert,
	IconTrash,
} from "@tabler/icons-react";
import { DebugData } from "@components/debug-data";
import { UnifiedMod } from "@hooks/use-unified-mods";
import { ItemName } from "@components/item-name";
import { getModTitle } from "@util/game-mod";
import { DeprecatedBadge } from "./deprecated-badge";

const { deleteMod, downloadMod, openModFolder, runRunnableWithoutGame } =
	commands;

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
				<Group>
					<ItemName label={`by ${props.mod.remote?.author}`}>
						{getModTitle(props.mod)}
					</ItemName>
					{props.mod.remote?.deprecated && <DeprecatedBadge />}
				</Group>
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
				{localVersion && (
					<CommandButton
						color="red"
						variant="light"
						confirmationText="You sure? Any files inside the mod's folder will be lost."
						leftSection={<IconTrash />}
						onClick={() => deleteMod(props.mod.common.id)}
					>
						Delete mod
					</CommandButton>
				)}
				<DebugData data={props.mod} />
			</Stack>
		</Modal>
	);
}
