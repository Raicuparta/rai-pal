import { commands, InstalledGame, ProviderId } from "@api/bindings";
import { Table } from "@mantine/core";
import { useEffect, useState } from "react";
import { installedGamesColumns } from "./installed-games-columns";
import React from "react";
import { useSetAtom } from "jotai";
import { selectedInstalledGameAtom } from "./selected-installed-game";
import { InstalledGameId } from "./installed-games-page";

type Props = {
	readonly item: InstalledGameId;
};

export function InstalledGameRow(props: Props) {
	const [game, setGame] = useState<InstalledGame>();
	const setSelectedGame = useSetAtom(selectedInstalledGameAtom);

	useEffect(() => {
		commands
			.getInstalledGame(props.item.provider, props.item.id)
			.then((result) => {
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
