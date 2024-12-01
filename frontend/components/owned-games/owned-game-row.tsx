import { Table } from "@mantine/core";
import { ownedGamesColumns } from "./owned-games-columns";
import React from "react";
import { useAtom } from "jotai";
import { selectedOwnedGameAtom } from "./selected-owned-game";
import { OwnedGameId } from "./owned-games-page";
import { useOwnedGame } from "@hooks/use-owned-game";
import { OwnedGameModal } from "./owned-game-modal";
import { ItemProps } from "react-virtuoso";

export const OwnedGameRow = React.forwardRef(function OwnedGameRow(
	props: ItemProps<OwnedGameId>,
	ref: React.ForwardedRef<HTMLTableRowElement>,
) {
	const game = useOwnedGame(props.item.provider, props.item.id);
	const [selectedGame, setSelectedGame] = useAtom(selectedOwnedGameAtom);

	return (
		<>
			{game &&
				selectedGame &&
				selectedGame.provider === game.provider &&
				selectedGame.id === game.providerGameId && (
					<OwnedGameModal game={game} />
				)}
			<Table.Tr
				ref={ref}
				onClick={() =>
					game &&
					setSelectedGame({
						id: game.providerGameId,
						provider: game.provider,
					})
				}
			>
				{ownedGamesColumns.map((column) => (
					<React.Fragment key={column.id}>
						{game ? column.renderCell(game) : <Table.Td>...</Table.Td>}
					</React.Fragment>
				))}
			</Table.Tr>
		</>
	);
});
