import { AppType, EngineBrand, GameMode, ProviderId } from "@api/bindings";
import {
	AppTypeBadge,
	EngineBadge,
	GameModeBadge,
	ProviderBadge,
} from "@components/badges/color-coded-badge";
import { TableColumnBase, columnMapToList } from "@components/table/table-head";
import { Table } from "@mantine/core";
import { IconCheck } from "@tabler/icons-react";
import styles from "../table/table.module.css";
import { ThumbnailCell } from "@components/table/thumbnail-cell";
import {
	appTypeFilterOptions,
	engineFilterOptions,
	providerFilterOptions,
} from "../../util/common-filter-options";
import { getThumbnailWithFallback } from "../../util/fallback-thumbnail";
import { sortGamesByEngine } from "../../util/game-engines";
import { ProcessedOwnedGame } from "@hooks/use-processed-owned-games";

const thumbnail: TableColumnBase<ProcessedOwnedGame> = {
	label: "Thumbnail",
	hideLabel: true,
	hidable: true,
	hideInDetails: true,
	width: 100,
	renderCell: (game) => (
		<ThumbnailCell
			src={getThumbnailWithFallback(game.thumbnailUrl, game.provider)}
		/>
	),
};

const name: TableColumnBase<ProcessedOwnedGame> = {
	label: "Game",
	width: undefined,
	hideInDetails: true,
	getSortValue: (game) => game.name,
	renderCell: (game) => (
		<Table.Td className={styles.nameCell}>{game.name}</Table.Td>
	),
};

const provider: TableColumnBase<ProcessedOwnedGame, ProviderId> = {
	label: "Provider",
	width: 110,
	center: true,
	hidable: true,
	getSortValue: (game) => game.provider,
	getFilterValue: (game) => game.provider,
	filterOptions: providerFilterOptions,
	unavailableValues: ["Manual"],
	renderCell: (game) => (
		<Table.Td>
			<ProviderBadge value={game.provider} />
		</Table.Td>
	),
};

const appType: TableColumnBase<ProcessedOwnedGame, AppType> = {
	label: "App Type",
	width: 110,
	center: true,
	hidable: true,
	getSortValue: (game) => game.appType,
	getFilterValue: (game) => game.appType,
	filterOptions: appTypeFilterOptions,
	renderCell: (game) => (
		<Table.Td>
			<AppTypeBadge value={game.appType}>{game.appType}</AppTypeBadge>
		</Table.Td>
	),
};

function getEngine(game: ProcessedOwnedGame) {
	return (
		// The remote game database can have multiple engines assigned to a game.
		// Usually that's for games that had their engine versions change during the game's lifespan.
		// For now we're just picking the latest one and displaying that the table,
		// but maybe later we'll want to somehow display all versions,
		// because for some games you can get an older version that's compatible with more mods.
		game.remoteData?.engines?.sort(sortGamesByEngine)?.reverse()?.[0] ?? null
	);
}

const engine: TableColumnBase<ProcessedOwnedGame, EngineBrand> = {
	label: "Engine",
	width: 150,
	center: true,
	hidable: true,
	sort: (dataA, dataB) => sortGamesByEngine(getEngine(dataA), getEngine(dataB)),
	getFilterValue: (game) => getEngine(game)?.brand ?? null,
	filterOptions: engineFilterOptions,
	renderCell: (game) => (
		<Table.Td>
			<EngineBadge
				maw={70}
				value={getEngine(game)?.brand}
				label={getEngine(game)?.version?.display ?? "-"}
			/>
		</Table.Td>
	),
};

const installed: TableColumnBase<ProcessedOwnedGame, string> = {
	label: "Installed",
	width: 60,
	center: true,
	hidable: true,
	getSortValue: (game) => game.isInstalled,
	getFilterValue: (game) => `${game.isInstalled}`,
	filterOptions: [
		{ label: "Installed", value: "true" },
		{ label: "Not Installed", value: "false" },
	],
	renderCell: (game) => (
		<Table.Td align="center">{game.isInstalled ? <IconCheck /> : ""}</Table.Td>
	),
};

const gameMode: TableColumnBase<ProcessedOwnedGame, GameMode> = {
	label: "Mode",
	width: 90,
	center: true,
	hidable: true,
	getSortValue: (game) => game.gameMode,
	getFilterValue: (game) => game.gameMode,
	filterOptions: [
		{ label: "Flat", value: "Flat" },
		{ label: "VR", value: "VR" },
	],
	renderCell: (game) => (
		<Table.Td>
			<GameModeBadge value={game.gameMode} />
		</Table.Td>
	),
};

const releaseDate: TableColumnBase<ProcessedOwnedGame> = {
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
	appType,
	gameMode,
	installed,
	releaseDate,
};

export type OwnedGameColumnsId = keyof typeof ownedGamesColumnsMap;

export const ownedGamesColumns = columnMapToList(ownedGamesColumnsMap);
