import { Group, Modal, Stack } from "@mantine/core";
import { ModalImage } from "@components/modal-image";
import { DebugData } from "@components/debug-data";
import { ItemName } from "@components/item-name";
import { getThumbnailWithFallback } from "@util/fallback-thumbnail";
import { ProviderCommandButtons } from "../providers/provider-command-dropdown";
import { OwnedGame } from "@api/bindings";
import { useSetAtom } from "jotai";
import { selectedInstalledGameAtom } from "@components/installed-games/installed-games-state";

type Props = {
	readonly game: OwnedGame;
};

export function OwnedGameModal(props: Props) {
	const setSelectedGame = useSetAtom(selectedInstalledGameAtom);

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
				{/* TODO probably gonna move all this into a single modal anyway */}
				{/* <TableItemDetails
					columns={ownedGamesColumns}
					item={props.game}
				/> */}
				<ProviderCommandButtons
					game={props.game}
					isInstalled={false} // TODO
				/>
				<DebugData data={props.game} />
			</Stack>
		</Modal>
	);
}
