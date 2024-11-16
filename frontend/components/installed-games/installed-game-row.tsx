import { commands, InstalledGame, ProviderId } from "@api/bindings";
import { Table } from "@mantine/core";
import { useEffect, useState } from "react";
import { installedGamesColumns } from "./installed-games-columns";
import React from "react";

type Props = {
	readonly game_id: string;
	readonly provider_id: ProviderId;
};

export function InstalledGameRow(props: Props) {
	const [game, setGame] = useState<InstalledGame>();

	useEffect(() => {
		commands
			.getInstalledGame(props.provider_id, props.game_id)
			.then((result) => {
				if (result.status === "ok") {
					setGame(result.data);
				}
			});
	}, [props.game_id, props.provider_id]);

	return (
		<>
			{game ? (
				installedGamesColumns.map((column) => (
					<React.Fragment key={column.id}>
						{column.renderCell(game)}
					</React.Fragment>
				))
			) : (
				<Table.Td>...</Table.Td>
			)}
		</>
	);
}
