import { Table } from "@mantine/core";
import React from "react";
import { useAtom } from "jotai";
import { selectedGameAtom } from "./games-state";
import { useGame } from "@hooks/use-game";
import { GameModal } from "./game-modal";
import { ItemProps } from "react-virtuoso";
import { DbGame, ProviderId } from "@api/bindings";
import { gamesColumns } from "./games-columns";
import { useAppSettings } from "@hooks/use-app-settings";

// Needs to be consistent with height set in table.module.css ugh. TODO: fix that.
export const gameRowHeight = 60;

export const GameRow = React.forwardRef(function GameRow(
	props: ItemProps<[ProviderId, string]>,
	ref: React.ForwardedRef<HTMLTableRowElement>,
) {
	const game = useGame(props.item[0], props.item[1]);
	const [selectedGame, setSelectedGame] = useAtom(selectedGameAtom);

	const isSelected =
		!!game &&
		!!selectedGame &&
		selectedGame[1] === game.gameId &&
		selectedGame[0] == game.providerId;

	return (
		<>
			{isSelected && <GameModal game={game} />}
			<GameRowInner
				game={game}
				ref={ref}
				onClick={() =>
					setSelectedGame([game.providerId, game.gameId])
				}
			/>
		</>
	);
});

type Props = { readonly game: DbGame; readonly onClick?: () => void };

export const GameRowInner = React.forwardRef(function GameRowInner(
	props: Props,
	ref: React.ForwardedRef<HTMLTableRowElement>,
) {
	const [settings] = useAppSettings();

	return (
		<Table.Tr
			ref={ref}
			onClick={props.onClick}
		>
			{gamesColumns.map(
				(column) =>
					(!settings.hideGameThumbnails || column.id !== "thumbnail") && (
						<React.Fragment key={column.id}>
							<column.component item={props.game} />
						</React.Fragment>
					),
			)}
		</Table.Tr>
	);
});
