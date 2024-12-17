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
import styles from "./game-row.module.css";

export const InstalledGameRow = React.forwardRef(function InstalledGameRow(
	props: ItemProps<GameId>,
	ref: React.ForwardedRef<HTMLTableRowElement>,
) {
	const game = useGame(props.item.providerId, props.item.index);
	const [selectedGame, setSelectedGame] = useAtom(selectedInstalledGameAtom);

	const columns = useVisibleInstalledGameColumns();

	const isSelected =
		!!game &&
		!!selectedGame &&
		selectedGame.gameId === game.id &&
		selectedGame.providerId == game.providerId;

	return (
		<>
			{isSelected && <InstalledGameModal game={game} />}
			<Table.Tr
				ref={ref}
				className={game?.installedGame ? styles.installed : styles.owned}
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
						<column.component item={game} />
					</React.Fragment>
				))}
			</Table.Tr>
		</>
	);
});
