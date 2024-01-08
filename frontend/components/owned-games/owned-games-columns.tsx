import {
	GameEngineBrand,
	GameMode,
	OwnedGame,
	ProviderId,
	UevrScore,
} from "@api/bindings";
import {
	EngineBadge,
	GameModeBadge,
	ProviderBadge,
	UevrScoreBadge,
} from "@components/badges/color-coded-badge";
import { TableColumnBase, columnMapToList } from "@components/table/table-head";
import { Table } from "@mantine/core";
import { IconCheck } from "@tabler/icons-react";
import styles from "../table/table.module.css";
import { ThumbnailCell } from "@components/table/thumbnail-cell";
import {
	engineFilterOptions,
	providerFilterOptions,
} from "../../util/common-filter-options";
import { getThumbnailWithFallback } from "../../util/fallback-thumbnail";
import { sortGamesByEngine } from "../../util/game-engines";

const thumbnail: TableColumnBase<OwnedGame> = {
	label: "Thumbnail",
	hideLabel: true,
	hidable: true,
	hideInDetails: true,
	width: 100,
	renderCell: (game) => (
		<ThumbnailCell
			src={getThumbnailWithFallback(game.thumbnailUrl, game.providerId)}
		/>
	),
};

const name: TableColumnBase<OwnedGame> = {
	label: "Game",
	width: undefined,
	hideInDetails: true,
	getSortValue: (game) => game.name,
	renderCell: (game) => (
		<Table.Td className={styles.nameCell}>{game.name}</Table.Td>
	),
};

const provider: TableColumnBase<OwnedGame, ProviderId> = {
	label: "Provider",
	width: 110,
	center: true,
	hidable: true,
	getSortValue: (game) => game.providerId,
	filterOptions: providerFilterOptions,
	unavailableValues: ["Manual", "Xbox"],
	renderCell: (game) => (
		<Table.Td>
			<ProviderBadge value={game.providerId} />
		</Table.Td>
	),
};

const engine: TableColumnBase<OwnedGame, GameEngineBrand> = {
	label: "Engine",
	width: 120,
	center: true,
	hidable: true,
	sort: (dataA, dataB) => sortGamesByEngine(dataA.engine, dataB.engine),
	getFilterValue: (game) => game.engine?.brand ?? "",
	unavailableValues: ["Godot"],
	filterOptions: engineFilterOptions,
	renderCell: ({ engine }) => (
		<Table.Td>
			<EngineBadge
				miw={70}
				value={engine?.brand}
				label={engine ? engine.version?.display ?? "-" : undefined}
			/>
		</Table.Td>
	),
};

const installed: TableColumnBase<OwnedGame, string> = {
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

const gameMode: TableColumnBase<OwnedGame, GameMode> = {
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

const uevrScore: TableColumnBase<OwnedGame, UevrScore> = {
	label: "UEVR",
	width: 90,
	center: true,
	hidable: true,
	getSortValue: (game) => game.uevrScore,
	filterOptions: [
		{ label: "Any UEVR score", value: "" },
		{ label: "A", value: "A" },
		{ label: "B", value: "B" },
		{ label: "C", value: "C" },
		{ label: "D", value: "D" },
	],
	renderCell: (game) => (
		<Table.Td>
			<UevrScoreBadge value={game.uevrScore} />
		</Table.Td>
	),
};

const releaseDate: TableColumnBase<OwnedGame> = {
	label: "Release Date",
	width: 130,
	center: true,
	hidable: true,
	getSortValue: (game) => game.releaseDate,
	renderCell: (game) => (
		<Table.Td align="center">
			{game.releaseDate
				? new Date(Number(game.releaseDate) * 1000).toISOString().split("T")[0]
				: "Unknown"}
		</Table.Td>
	),
};

const ownedGamesColumnsMap = {
	thumbnail,
	name,
	provider,
	engine,
	gameMode,
	uevrScore,
	installed,
	releaseDate,
};

export type OwnedGameColumnsId = keyof typeof ownedGamesColumnsMap;

export const ownedGamesColumns = columnMapToList(ownedGamesColumnsMap);
