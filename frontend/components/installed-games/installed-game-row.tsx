import { commands, InstalledGame, ProviderId } from "@api/bindings";
import { Table } from "@mantine/core";
import { useEffect, useState } from "react";
import { installedGamesColumns } from "./installed-games-columns";
import { useTableRowContent } from "@components/table/use-table-row-content";
import { ThumbnailCell } from "@components/table/thumbnail-cell";
import { getThumbnailWithFallback } from "@util/fallback-thumbnail";

type Props = {
	readonly game_id: string;
	readonly provider_id: ProviderId;
};

export function InstalledGameRow(props: Props) {
	const [game, setGame] = useState<InstalledGame>();

	const itemContent = useTableRowContent(installedGamesColumns);

	useEffect(() => {
		commands
			.getInstalledGame(props.provider_id, props.game_id)
			.then((result) => {
				if (result.status === "ok") {
					setGame(result.data);
				}
			});
	}, [props.game_id, props.provider_id]);

	if (!game) {
		return (
			<>
				<Table.Td></Table.Td>
				<Table.Td>...</Table.Td>
			</>
		);
	}

	return (
		<>
			<ThumbnailCell
				src={getThumbnailWithFallback(game.thumbnailUrl, props.provider_id)}
			/>
			<Table.Td>{game.title.display}</Table.Td>
			<Table.Td></Table.Td>
			<Table.Td>{game.provider}</Table.Td>
			<Table.Td>{game.executable.architecture}</Table.Td>
			<Table.Td>{game.executable.scriptingBackend}</Table.Td>
			<Table.Td>
				{game.executable.engine?.brand}{" "}
				{game.executable.engine?.version?.display}
			</Table.Td>
		</>
	);
}
