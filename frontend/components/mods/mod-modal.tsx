import { Modal, Stack } from "@mantine/core";
import { GameMod, downloadMod, openModFolder } from "@api/bindings";
import { CommandButton } from "@components/command-button";
import { IconDownload, IconFolderCog } from "@tabler/icons-react";
import { CommandButtonGroup } from "@components/command-button-group";
import { DebugData } from "@components/debug-data";

type Props = {
	readonly mod: GameMod;
	readonly onClose: () => void;
};

export function ModModal(props: Props) {
	return (
		<Modal
			centered
			onClose={props.onClose}
			opened
			size="lg"
			title={props.mod.remoteMod?.title ?? props.mod.common.id}
		>
			<Stack>
				<CommandButtonGroup label="Mod Actions">
					<CommandButton
						leftSection={<IconFolderCog />}
						onClick={() => openModFolder("bepinex", props.mod.common.id)} // TODO modloader id
					>
						Open mod folder
					</CommandButton>
					<CommandButton
						leftSection={<IconDownload />}
						onClick={() => downloadMod("bepinex", props.mod.common.id)} // TODO modloader id
					>
						Download mod
					</CommandButton>
				</CommandButtonGroup>
				<DebugData data={props.mod} />
			</Stack>
		</Modal>
	);
}
