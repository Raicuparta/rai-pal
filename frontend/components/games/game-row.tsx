import React from "react";
import { useAtom } from "jotai";
import { selectedGameAtom } from "./games-state";
import { useGame } from "@hooks/use-game";
import { GameModal } from "./game-modal";
import { ItemProps } from "react-virtuoso";
import { Game, GameId } from "@api/bindings";
import { gamesColumns } from "./games-columns";
import { css } from "@styled-system/css";

export const gameRowHeight = 76;

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
		<tr
			ref={ref}
			onClick={props.onClick}
			className={css({
				height: gameRowHeight,
				maxHeight: gameRowHeight,
				minHeight: gameRowHeight,
				backgroundColor: "dark.600",
				borderStyle: "solid",
				borderBottomWidth: 2,
				borderColor: "dark.700",
				cursor: props.onClick && "pointer",
				_hover: props.onClick && {
					backgroundColor: "dark.500",
				},
			})}
		>
			{gamesColumns.map((column) => (
				<React.Fragment key={column.id}>
					<column.component item={props.game} />
				</React.Fragment>
			))}
		</tr>
	);
});
