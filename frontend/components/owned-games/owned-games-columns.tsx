import {
	GameEngineBrand,
	GameMode,
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
	filterOptions: providerFilterOptions,
	unavailableValues: ["Manual", "Xbox"],
	renderCell: (game) => (
		<Table.Td>
			<ProviderBadge value={game.provider} />
		</Table.Td>
	),
};

const engine: TableColumnBase<ProcessedOwnedGame, GameEngineBrand> = {
	label: "Engine",
	width: 150,
	center: true,
	hidable: true,
	sort: (dataA, dataB) =>
		sortGamesByEngine(dataA.remoteData?.engine, dataB.remoteData?.engine),
	getFilterValue: (game) => game.remoteData?.engine?.brand ?? "",
	filterOptions: engineFilterOptions,
	renderCell: ({ remoteData }) => (
		<Table.Td>
			<EngineBadge
				maw={70}
				value={remoteData?.engine?.brand}
				label={remoteData?.engine?.version?.display ?? "-"}
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
		{ label: "Any install state", value: "" },
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

const uevrScore: TableColumnBase<ProcessedOwnedGame, UevrScore> = {
	label: "UEVR",
	width: 90,
	center: true,
	hidable: true,
	getSortValue: (game) => game.remoteData?.uevrScore,
	filterOptions: [
		{ label: "A", value: "A" },
		{ label: "B", value: "B" },
		{ label: "C", value: "C" },
		{ label: "D", value: "D" },
	],
	renderCell: (game) => (
		<Table.Td>
			<UevrScoreBadge value={game.remoteData?.uevrScore} />
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
	gameMode,
	uevrScore,
	installed,
	releaseDate,
};

export type OwnedGameColumnsId = keyof typeof ownedGamesColumnsMap;

export const ownedGamesColumns = columnMapToList(ownedGamesColumnsMap);
