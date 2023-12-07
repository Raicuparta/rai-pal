import { Modal, Stack } from "@mantine/core";
import { GameMod, openModFolder } from "@api/bindings";
import { useMemo } from "react";
import { CommandButton } from "@components/command-button";
import { IconFolderCog } from "@tabler/icons-react";
import { CodeHighlight } from "@mantine/code-highlight";
import { CommandButtonGroup } from "@components/command-button-group";

type Props = {
	readonly mod: GameMod;
	readonly onClose: () => void;
};

export function ModModal(props: Props) {
	const debugData = useMemo(
		() => JSON.stringify(props.mod, null, 2),
		[props.mod],
	);

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
						Open Mod Folder
					</CommandButton>
				</CommandButtonGroup>
				<Stack gap="xs">
					<label>Debug Data</label>
					<CodeHighlight
						code={debugData}
						language="json"
					/>
				</Stack>
			</Stack>
		</Modal>
	);
}
