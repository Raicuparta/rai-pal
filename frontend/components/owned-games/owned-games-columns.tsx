import { OwnedGame } from "@api/bindings";
import { EngineBadge } from "@components/badges/color-coded-badge";
import { TableColumn } from "@components/table/table-head";
import { Flex, Table } from "@mantine/core";
import { IconCheck } from "@tabler/icons-react";
import styles from "../table/table.module.css";
import { ThumbnailCell } from "@components/table/thumbnail-cell";

export const ownedGamesColumns: TableColumn<OwnedGame>[] = [
	{
		id: "thumbnailUrl",
		label: "",
		width: 100,
		renderCell: (game) => <ThumbnailCell url={game.thumbnailUrl} />,
	},
	{
		id: "name",
		label: "Game",
		width: undefined,
		getSortValue: (game) => game.name,
		renderCell: (game) => (
			<Table.Td className={styles.leftAligned}>
				<Flex>{game.name}</Flex>
			</Table.Td>
		),
	},
	{
		id: "engine",
		label: "Engine",
		width: 100,
		center: true,
		hidable: true,
		getSortValue: (game) => game.engine,
		renderCell: (game) => (
			<Table.Td align="center">
				<EngineBadge value={game.engine} />
			</Table.Td>
		),
	},
	{
		id: "installed",
		label: "Installed",
		width: 60,
		center: true,
		hidable: true,
		getSortValue: (game) => game.installed,
		renderCell: (game) => (
			<Table.Td align="center">{game.installed ? <IconCheck /> : ""}</Table.Td>
		),
	},
	{
		id: "releaseDate",
		label: "Release Date",
		width: 130,
		center: true,
		hidable: true,
		getSortValue: (game) => game.releaseDate,
		renderCell: (game) => (
			<Table.Td align="center">
				{game.releaseDate
					? new Date(game.releaseDate * 1000).toISOString().split("T")[0]
					: "Unknown"}
			</Table.Td>
		),
	},
];
