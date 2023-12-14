import {
	GameEngineBrand,
	GameMode,
	OwnedGame,
	ProviderId,
} from "@api/bindings";
import {
	EngineBadge,
	GameModeBadge,
	ProviderBadge,
} from "@components/badges/color-coded-badge";
import { TableColumn } from "@components/table/table-head";
import { Table } from "@mantine/core";
import { IconCheck } from "@tabler/icons-react";
import styles from "../table/table.module.css";
import { ThumbnailCell } from "@components/table/thumbnail-cell";
import {
	engineFilterOptions,
	providerFilterOptions,
} from "../../util/common-filter-options";

const thumbnailColumn: TableColumn<OwnedGame> = {
	id: "thumbnail",
	label: "Thumbnail",
	hideLabel: true,
	hidable: true,
	hideInDetails: true,
	width: 100,
	renderCell: (game) => <ThumbnailCell url={game.thumbnailUrl} />,
};

const nameColumn: TableColumn<OwnedGame> = {
	id: "name",
	label: "Game",
	width: undefined,
	hideInDetails: true,
	getSortValue: (game) => game.name,
	renderCell: (game) => (
		<Table.Td className={styles.nameCell}>{game.name}</Table.Td>
	),
};

const providerColumn: TableColumn<OwnedGame, ProviderId> = {
	id: "provider",
	label: "Provider",
	width: 110,
	center: true,
	hidable: true,
	getSortValue: (game) => game.providerId,
	filterOptions: providerFilterOptions,
	renderCell: (game) => (
		<Table.Td>
			<ProviderBadge value={game.providerId} />
		</Table.Td>
	),
};

const engineColumn: TableColumn<OwnedGame, GameEngineBrand> = {
	id: "engine",
	label: "Engine",
	width: 100,
	center: true,
	hidable: true,
	getSortValue: (game) => game.engine,
	filterOptions: engineFilterOptions,
	renderCell: (game) => (
		<Table.Td align="center">
			<EngineBadge value={game.engine} />
		</Table.Td>
	),
};

const installedColumn: TableColumn<OwnedGame, string> = {
	id: "installed",
	label: "Installed",
	width: 60,
	center: true,
	hidable: true,
	getSortValue: (game) => game.installed,
	getFilterValue: (game) => `${game.installed}`,
	filterOptions: [
		{ label: "Any install state", value: "" },
		{ label: "Installed", value: "true" },
		{ label: "Not Installed", value: "false" },
	],
	renderCell: (game) => (
		<Table.Td align="center">{game.installed ? <IconCheck /> : ""}</Table.Td>
	),
};

const gameModeColumn: TableColumn<OwnedGame, GameMode> = {
	id: "mode",
	label: "Mode",
	width: 90,
	center: true,
	hidable: true,
	getSortValue: (game) => game.gameMode,
	filterOptions: [
		{ label: "Any mode", value: "" },
		{ label: "Flat", value: "Flat" },
		{ label: "VR", value: "VR" },
	],
	renderCell: (game) => (
		<Table.Td>
			<GameModeBadge value={game.gameMode} />
		</Table.Td>
	),
};

const releaseDateColumn: TableColumn<OwnedGame> = {
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
};

export const ownedGamesColumns: TableColumn<OwnedGame>[] = [
	thumbnailColumn,
	nameColumn,
	providerColumn,
	engineColumn,
	gameModeColumn,
	installedColumn,
	releaseDateColumn,
];
