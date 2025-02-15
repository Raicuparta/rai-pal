import { Table } from "@mantine/core";
import React from "react";
import { useAtom } from "jotai";
import { selectedGameAtom } from "./games-state";
import { useGame } from "@hooks/use-game";
import { GameModal } from "./game-modal";
import { ItemProps } from "react-virtuoso";
import { Game, GameId } from "@api/bindings";
import { gamesColumns } from "./games-columns";

// Needs to be consistent with height set in table.module.css ugh. TODO: fix that.
export const gameRowHeight = 60;

export const GameRow = React.forwardRef(function GameRow(
	props: ItemProps<GameId>,
	ref: React.ForwardedRef<HTMLTableRowElement>,
) {
	const game = useGame(props.item);
	const [selectedGame, setSelectedGame] = useAtom(selectedGameAtom);

	const isSelected =
		!!game &&
		!!selectedGame &&
		selectedGame.gameId === game.id.gameId &&
		selectedGame.providerId == game.id.providerId;

	return (
		<>
			{isSelected && <GameModal game={game} />}
			<GameRowInner
				game={game}
				ref={ref}
				onClick={() => game && setSelectedGame(game.id)}
			/>
		</>
	);
});

type Props = { readonly game: Game; readonly onClick?: () => void };

export const GameRowInner = React.forwardRef(function GameRowInner(
	props: Props,
	ref: React.ForwardedRef<HTMLTableRowElement>,
) {
	return (
		<Table.Tr
			ref={ref}
			onClick={props.onClick}
		>
			{gamesColumns.map((column) => (
				<React.Fragment key={column.id}>
					<column.component item={props.game} />
				</React.Fragment>
			))}
		</Table.Tr>
	);
});
