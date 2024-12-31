import { Table } from "@mantine/core";
import React from "react";
import { useAtom } from "jotai";
import { selectedGameAtom } from "./games-state";
import { useGame } from "@hooks/use-game";
import { GameModal } from "./game-modal";
import { ItemProps } from "react-virtuoso";
import { GameId } from "@api/bindings";
import { gamesColumns } from "./games-columns";

export const gameRowHeight = 76;

export const GameRow = React.forwardRef(function GameRow(
	props: ItemProps<GameId>,
	ref: React.ForwardedRef<HTMLTableRowElement>,
) {
	const game = useGame(props.item.providerId, props.item.gameId);
	const [selectedGame, setSelectedGame] = useAtom(selectedGameAtom);

	const isSelected =
		!!game &&
		!!selectedGame &&
		selectedGame.gameId === game.uniqueId &&
		selectedGame.providerId == game.providerId;

	return (
		<>
			{isSelected && <GameModal game={game} />}
			<Table.Tr
				ref={ref}
				h={gameRowHeight}
				onClick={() =>
					game &&
					setSelectedGame({
						gameId: game.uniqueId,
						providerId: game.providerId,
					})
				}
			>
				{gamesColumns.map((column) => (
					<React.Fragment key={column.id}>
						<column.component item={game} />
					</React.Fragment>
				))}
			</Table.Tr>
		</>
	);
});
