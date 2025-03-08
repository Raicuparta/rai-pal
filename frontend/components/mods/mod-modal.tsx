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
import { useLocalization } from "@hooks/use-localization";

type Props = {
	readonly mod: UnifiedMod;
	readonly onClose: () => void;
};

export function ModModal(props: Props) {
	const t = useLocalization("modModal");
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
						onClick={() => commands.runRunnableWithoutGame(props.mod.common.id)}
					>
						{t("runMod")}
					</CommandButton>
				)}
				{props.mod.local && (
					<CommandButton
						leftSection={<IconFolderCog />}
						onClick={() => commands.openModFolder(props.mod.common.id)}
					>
						{t("openModFolder")}
					</CommandButton>
				)}
				{isDownloadAvailable && (
					<CommandButton
						leftSection={isOutdated ? <IconRefreshAlert /> : <IconDownload />}
						onClick={() => commands.downloadMod(props.mod.common.id)}
					>
						{isOutdated ? t("updateMod") : t("downloadMod")}
					</CommandButton>
				)}
				{localVersion && (
					<CommandButton
						color="red"
						variant="light"
						confirmationText={t("deleteModConfirmation")}
						leftSection={<IconTrash />}
						onClick={() => commands.deleteMod(props.mod.common.id)}
					>
						{t("deleteMod")}
					</CommandButton>
				)}
				<DebugData data={props.mod} />
			</Stack>
		</Modal>
	);
}
