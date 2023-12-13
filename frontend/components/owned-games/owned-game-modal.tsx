import { Modal, Stack } from "@mantine/core";
import { OwnedGame } from "@api/bindings";
import { CommandButton } from "@components/command-button";
import { IconBooks, IconBrowser, IconDownload } from "@tabler/icons-react";
import { steamCommands } from "../../util/steam";
import { ModalImage } from "@components/modal-image";
import { CommandButtonGroup } from "@components/command-button-group";
import { DebugData } from "@components/debug-data";
import { TableItemDetails } from "@components/table/table-item-details";
import { ownedGamesColumns } from "./owned-games-columns";

type Props = {
	readonly game: OwnedGame;
	readonly onClose: () => void;
};

export function OwnedGameModal(props: Props) {
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
				<CommandButtonGroup label="Game Actions">
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
				<TableItemDetails
					columns={ownedGamesColumns}
					item={props.game}
				/>
				<DebugData data={props.game} />
			</Stack>
		</Modal>
	);
}
