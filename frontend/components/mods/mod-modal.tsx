import { Modal, Stack } from "@mantine/core";
import { downloadMod, openModFolder } from "@api/bindings";
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
					<CommandButton
						leftSection={<IconFolderCog />}
						onClick={() => openModFolder(props.mod.common.id)}
					>
						Open mod folder
					</CommandButton>
					{Boolean(props.mod.remote) && (
						<CommandButton
							leftSection={<IconDownload />}
							onClick={() =>
								downloadMod(props.mod.common.loaderId, props.mod.common.id)
							}
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
