import { Group, Modal, Stack } from "@mantine/core";
import { ModalImage } from "@components/modal-image";
import { DebugData } from "@components/debug-data";
import { TableItemDetails } from "@components/table/table-item-details";
import { ownedGamesColumns } from "./owned-games-columns";
import { ItemName } from "@components/item-name";
import { getThumbnailWithFallback } from "@util/fallback-thumbnail";
import { ProviderCommandButtons } from "../providers/provider-command-dropdown";
import { OwnedGame } from "@api/bindings";
import { selectedOwnedGameAtom } from "./selected-owned-game";
import { useSetAtom } from "jotai";

type Props = {
	readonly game: OwnedGame;
};

export function OwnedGameModal(props: Props) {
	const setSelectedGame = useSetAtom(selectedOwnedGameAtom);

	const close = () => setSelectedGame(null);

	return (
		<Modal
			centered
			onClose={close}
			opened
			size="xl"
			title={
				<Group>
					<ModalImage
						src={getThumbnailWithFallback(
							props.game.thumbnailUrl,
							props.game.provider,
						)}
					/>
					<ItemName>{props.game.title.display}</ItemName>
				</Group>
			}
		>
			<Stack>
				<TableItemDetails
					columns={ownedGamesColumns}
					item={props.game}
				/>
				<ProviderCommandButtons
					game={props.game}
					isInstalled={props.game.isInstalled}
				/>
				<DebugData data={props.game} />
			</Stack>
		</Modal>
	);
}
