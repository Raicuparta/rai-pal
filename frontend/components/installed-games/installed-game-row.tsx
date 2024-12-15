import { Table } from "@mantine/core";
import React from "react";
import { useAtom } from "jotai";
import {
	selectedInstalledGameAtom,
	useVisibleInstalledGameColumns,
} from "./installed-games-state";
import { useGame } from "@hooks/use-game";
import { InstalledGameModal } from "./installed-game-modal";
import { ItemProps } from "react-virtuoso";
import { GameId } from "@api/bindings";

export const InstalledGameRow = React.forwardRef(function InstalledGameRow(
	props: ItemProps<GameId>,
	ref: React.ForwardedRef<HTMLTableRowElement>,
) {
	const game = useGame(props.item.providerId, props.item.index);
	const [selectedGame, setSelectedGame] = useAtom(selectedInstalledGameAtom);

	const columns = useVisibleInstalledGameColumns();

	return (
		<>
			{game &&
				selectedGame &&
				selectedGame.gameId === game.id &&
				selectedGame.providerId == game.providerId &&
				game.installedGames[0] && (
					<InstalledGameModal game={game.installedGames[0]} />
				)}
			<Table.Tr
				ref={ref}
				onClick={() =>
					game &&
					setSelectedGame({
						gameId: game.id,
						providerId: game.providerId,
					})
				}
			>
				{columns.map((column) => (
					<React.Fragment key={column.id}>
						{game ? column.renderCell(game) : <Table.Td>...</Table.Td>}
					</React.Fragment>
				))}
			</Table.Tr>
		</>
	);
});
