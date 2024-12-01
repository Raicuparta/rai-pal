import {
	EngineBadge,
	ProviderBadge,
} from "@components/badges/color-coded-badge";
import { TableColumnBase, columnMapToList } from "@components/table/table-head";
import { Table } from "@mantine/core";
import { IconCheck } from "@tabler/icons-react";
import styles from "../table/table.module.css";
import { ThumbnailCell } from "@components/table/thumbnail-cell";
import { getThumbnailWithFallback } from "@util/fallback-thumbnail";
import { sortGamesByEngine } from "@util/game-engines";
import { ProcessedOwnedGame } from "@hooks/use-processed-owned-games";
import { renderGameTagsCell } from "@components/game-tags/game-tags";

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
	renderCell: (game) => (
		<Table.Td className={styles.nameCell}>{game.title.display}</Table.Td>
	),
};

const provider: TableColumnBase<ProcessedOwnedGame> = {
	label: "Provider",
	width: 110,
	center: true,
	hidable: true,
	renderCell: (game) => (
		<Table.Td>
			<ProviderBadge value={game.provider} />
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

const engine: TableColumnBase<ProcessedOwnedGame> = {
	label: "Engine",
	width: 150,
	center: true,
	hidable: true,
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

const gameTags: TableColumnBase<ProcessedOwnedGame> = {
	label: "Tags",
	width: 120,
	center: true,
	hidable: true,
	renderCell: renderGameTagsCell,
};

const installed: TableColumnBase<ProcessedOwnedGame> = {
	label: "Installed",
	width: 60,
	center: true,
	hidable: true,
	renderCell: (game) => (
		<Table.Td align="center">{game.isInstalled ? <IconCheck /> : ""}</Table.Td>
	),
};

const releaseDate: TableColumnBase<ProcessedOwnedGame> = {
	label: "Release Date",
	width: 130,
	center: true,
	hidable: true,
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
	gameTags,
	provider,
	engine,
	installed,
	releaseDate,
};

export type OwnedGameColumnsId = keyof typeof ownedGamesColumnsMap;

export const ownedGamesColumns = columnMapToList(ownedGamesColumnsMap);
