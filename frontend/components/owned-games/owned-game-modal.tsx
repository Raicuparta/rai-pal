import { Modal, Stack } from "@mantine/core";
import { OwnedGame } from "@api/bindings";
import { CommandButton } from "@components/command-button";
import { useMemo } from "react";
import { IconBooks, IconBrowser, IconDownload } from "@tabler/icons-react";
import { CodeHighlight } from "@mantine/code-highlight";
import { steamCommands } from "../../util/steam";
import { ModalImage } from "@components/modal-image";
import { CommandButtonGroup } from "@components/command-button-group";

type Props = {
	readonly game: OwnedGame;
	readonly onClose: () => void;
};

export function OwnedGameModal(props: Props) {
	const debugData = useMemo(
		() => JSON.stringify(props.game, null, 2),
		[props.game],
	);

	return (
		<Modal
			centered
			onClose={props.onClose}
			opened
			size="lg"
			title={props.game.name}
		>
			<Stack>
				<ModalImage src={props.game.thumbnailUrl} />
				<CommandButtonGroup
					label="Game Actions"
					m="auto"
				>
					<CommandButton
						leftSection={<IconBrowser />}
						onClick={() => steamCommands.openStorePage(props.game.id)}
					>
						Open Store Page
					</CommandButton>
					<CommandButton
						leftSection={<IconBooks />}
						onClick={() => steamCommands.showInLibrary(props.game.id)}
					>
						Show in Library
					</CommandButton>
					<CommandButton
						leftSection={<IconDownload />}
						onClick={() => steamCommands.install(props.game.id)}
					>
						Install
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
