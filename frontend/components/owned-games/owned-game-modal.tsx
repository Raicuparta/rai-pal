import { Button, Modal, Stack } from "@mantine/core";
import { OwnedGame } from "@api/bindings";
import { shell } from "@tauri-apps/api";
import { CommandButton } from "@components/command-button";
import { useMemo } from "react";
import { IconBrowser, IconDownload } from "@tabler/icons-react";
import { CodeHighlight } from "@mantine/code-highlight";

type Props = {
	readonly selectedGame: OwnedGame;
	readonly onClose: () => void;
};

export function OwnedGameModal(props: Props) {
	const debugData = useMemo(
		() => JSON.stringify(props.selectedGame, null, 2),
		[props.selectedGame],
	);

	return (
		<Modal
			centered
			onClose={props.onClose}
			opened
			title={props.selectedGame.name}
		>
			<Stack>
				<Button.Group orientation="vertical">
					<CommandButton
						leftSection={<IconBrowser />}
						onClick={() => shell.open(`steam://store/${props.selectedGame.id}`)}
					>
						Steam Store
					</CommandButton>
					<CommandButton
						leftSection={<IconDownload />}
						onClick={() =>
							shell.open(`steam://install/${props.selectedGame.id}`)
						}
					>
						Install
					</CommandButton>
				</Button.Group>
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
