import { commands, InstalledGame, ProviderId } from "@api/bindings";
import { Table } from "@mantine/core";
import { useEffect, useState } from "react";
import { installedGamesColumns } from "./installed-games-columns";
import React from "react";
import { InstalledGameTuple } from "./installed-games-page";
import { useSetAtom } from "jotai";
import { selectedInstalledGameAtom } from "./selected-installed-game";

type Props = {
	readonly item: InstalledGameTuple;
};

export function InstalledGameRow(props: Props) {
	const [game, setGame] = useState<InstalledGame>();
	const setSelectedGame = useSetAtom(selectedInstalledGameAtom);

	useEffect(() => {
		commands.getInstalledGame(props.item[0], props.item[1]).then((result) => {
			if (result.status === "ok") {
				setGame(result.data);
			}
		});
	}, [props.item]);

	return (
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
	);
}
