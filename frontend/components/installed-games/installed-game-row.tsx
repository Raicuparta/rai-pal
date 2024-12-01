import { Table } from "@mantine/core";
import React from "react";
import { useAtom } from "jotai";
import {
	selectedInstalledGameAtom,
	useVisibleInstalledGameColumns,
} from "./installed-games-state";
import { InstalledGameId } from "./installed-games-page";
import { useInstalledGame } from "@hooks/use-installed-game";
import { InstalledGameModal } from "./installed-game-modal";
import { ItemProps } from "react-virtuoso";

export const InstalledGameRow = React.forwardRef(function InstalledGameRow(
	props: ItemProps<InstalledGameId>,
	ref: React.ForwardedRef<HTMLTableRowElement>,
) {
	const game = useInstalledGame(props.item.provider, props.item.id);
	const [selectedGame, setSelectedGame] = useAtom(selectedInstalledGameAtom);

	const columns = useVisibleInstalledGameColumns();

	return (
		<>
			{game && selectedGame && selectedGame.id === game.id && (
				<InstalledGameModal game={game} />
			)}
			<Table.Tr
				ref={ref}
				onClick={() => game && setSelectedGame(game)}
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
