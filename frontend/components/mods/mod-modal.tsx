import { Box, Stack } from "@mantine/core";
import { commands } from "@api/bindings";
import { CommandButton } from "@components/command-button";
import {
	IconDownload,
	IconFolderCog,
	IconRefreshAlert,
	IconTrash,
} from "@tabler/icons-react";
import { DebugData } from "@components/debug-data";
import { UnifiedMod } from "@hooks/use-unified-mods";
import { useLocalization } from "@hooks/use-localization";
import { TableContainer } from "@components/table/table-container";
import { useMemo } from "react";
import { ModsTable } from "./mods-table";
import { SubPage } from "@components/sub-page";

type Props = {
	readonly mod: UnifiedMod;
	readonly onClose: () => void;
};

export function ModModal(props: Props) {
	const t = useLocalization("modModal");
	const isDownloadAvailable = Boolean(props.mod.remote?.download?.url);
	const localVersion = props.mod.local?.download?.id;
	const remoteVersion = props.mod.remote?.download?.id;
	const isOutdated =
		localVersion && remoteVersion && remoteVersion !== localVersion;

	const wrappedMod = useMemo(() => [props.mod], [props.mod]);

	return (
		<SubPage onClose={props.onClose}>
			<Box>
				<TableContainer>
					<ModsTable
						mods={wrappedMod}
						onClick={props.onClose}
					/>
				</TableContainer>
			</Box>
			<Stack
				p="xs"
				gap="xl"
			>
				<Stack>
					{props.mod.local && (
						<CommandButton
							leftSection={<IconFolderCog />}
							onClick={() => commands.openModFolder(props.mod.id)}
						>
							{t("openModFolder")}
						</CommandButton>
					)}
					{isDownloadAvailable && (
						<CommandButton
							leftSection={isOutdated ? <IconRefreshAlert /> : <IconDownload />}
							onClick={() => commands.downloadMod(props.mod.id)}
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
							onClick={() => commands.deleteMod(props.mod.id)}
						>
							{t("deleteMod")}
						</CommandButton>
					)}
					<DebugData data={props.mod} />
				</Stack>
			</Stack>
		</SubPage>
	);
}
