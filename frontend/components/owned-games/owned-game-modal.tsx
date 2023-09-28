import { Button, Code, CopyButton, Flex, Modal, Stack } from "@mantine/core";
import { OwnedGame } from "@api/bindings";
import { open } from "@tauri-apps/api/shell";
import { CommandButton } from "@components/command-button";
import { MdCopyAll, MdInstallDesktop, MdWeb } from "react-icons/md";
import { useMemo } from "react";

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
						icon={<MdWeb />}
						onClick={() =>
							open(`https://steampowered.com/app/${props.selectedGame.id}`)
						}
					>
						Open Steam Page
					</CommandButton>
					<CommandButton
						icon={<MdInstallDesktop />}
						onClick={() => open(`steam://install/${props.selectedGame.id}`)}
					>
						Install on Steam
					</CommandButton>
				</Button.Group>
				<Stack gap="xs">
					<Flex
						align="end"
						justify="space-between"
					>
						<label>Debug Data</label>
						<CopyButton value={debugData}>
							{({ copied, copy }) => (
								<Button
									color="green"
									leftSection={<MdCopyAll />}
									onClick={copy}
									size="xs"
									variant={copied ? "filled" : "default"}
								>
									Copy
								</Button>
							)}
						</CopyButton>
					</Flex>
					<Code block>{debugData}</Code>
				</Stack>
			</Stack>
		</Modal>
	);
}
