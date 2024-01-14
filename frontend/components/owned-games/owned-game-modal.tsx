import { Group, Modal, Stack } from "@mantine/core";
import {
	OwnedGame,
	installGame,
	openGamePage,
	showGameInLibrary,
} from "@api/bindings";
import { CommandButton } from "@components/command-button";
import { IconBooks, IconBrowser, IconDownload } from "@tabler/icons-react";
import { ModalImage } from "@components/modal-image";
import { CommandButtonGroup } from "@components/command-button-group";
import { DebugData } from "@components/debug-data";
import { TableItemDetails } from "@components/table/table-item-details";
import { ownedGamesColumns } from "./owned-games-columns";
import { ItemName } from "@components/item-name";
import { getThumbnailWithFallback } from "../../util/fallback-thumbnail";

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
			size="xl"
			title={
				<Group>
					<ModalImage
						src={getThumbnailWithFallback(
							props.game.thumbnailUrl,
							props.game.providerId,
						)}
					/>
					<ItemName>{props.game.name}</ItemName>
				</Group>
			}
		>
			<Stack>
				<TableItemDetails
					columns={ownedGamesColumns}
					item={props.game}
				/>
				{props.game.openPageCommand && (
					<CommandButton
						leftSection={<IconBrowser />}
						onClick={() => openGamePage(props.game.id)}
					>
						Open Store Page
					</CommandButton>
				)}
				{props.game.showLibraryCommand && (
					<CommandButton
						leftSection={<IconBooks />}
						onClick={() => showGameInLibrary(props.game.id)}
					>
						Show in Library
					</CommandButton>
				)}
				{props.game.installCommand && (
					<CommandButton
						leftSection={<IconDownload />}
						onClick={() => installGame(props.game.id)}
					>
						Install
					</CommandButton>
				)}

				<DebugData data={props.game} />
			</Stack>
		</Modal>
	);
}
