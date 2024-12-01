import { Table } from "@mantine/core";
import { installedGamesColumns } from "./installed-games-columns";
import React from "react";
import { useAtom } from "jotai";
import { selectedInstalledGameAtom } from "./selected-installed-game";
import { InstalledGameId } from "./installed-games-page";
import { useInstalledGame } from "@hooks/use-installed-game";
import { InstalledGameModal } from "./installed-game-modal";

type Props = {
	readonly item: InstalledGameId;
};

export function InstalledGameRow(props: Props) {
	const game = useInstalledGame(props.item.provider, props.item.id);
	const [selectedGame, setSelectedGame] = useAtom(selectedInstalledGameAtom);

	return (
		<>
			{game && selectedGame && selectedGame.id === game.id && (
				<InstalledGameModal game={game} />
			)}
			<Table.Tr onClick={() => game && setSelectedGame(game)}>
				{game ? (
					installedGamesColumns.map((column) => (
						<React.Fragment key={column.id}>
							{column.renderCell(game)}
						</React.Fragment>
					))
				) : (
					<Table.Td>...</Table.Td>
				)}
			</Table.Tr>
		</>
	);
}
