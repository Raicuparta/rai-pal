import { Table, Tooltip } from "@mantine/core";
import { TableColumnBase, columnMapToList } from "@components/table/table-head";
import { ItemName } from "../item-name";
import {
	ArchitectureBadge,
	EngineBadge,
	ProviderBadge,
	UnityBackendBadge,
} from "@components/badges/color-coded-badge";
import { ThumbnailCell } from "@components/table/thumbnail-cell";
import { OutdatedMarker } from "@components/outdated-marker";
import styles from "../table/table.module.css";
import { ProcessedInstalledGame } from "@hooks/use-processed-installed-games";
import { getThumbnailWithFallback } from "@util/fallback-thumbnail";
import { sortGamesByEngine } from "@util/game-engines";
import {
	getGameTagsSortValue,
	renderGameTagsCell,
} from "@components/game-tags/game-tags";

const thumbnail: TableColumnBase<ProcessedInstalledGame> = {
	hideInDetails: true,
	label: "Thumbnail",
	hideLabel: true,
	hidable: true,
	width: 100,
	renderCell: (game) => (
		<ThumbnailCell
			src={getThumbnailWithFallback(
				game.thumbnailUrl || game.ownedGame?.thumbnailUrl,
				game.provider,
			)}
		/>
	),
};

const name: TableColumnBase<ProcessedInstalledGame> = {
	hideInDetails: true,
	label: "Game",
	getSortValue: (game) => game.title.display,
	renderCell: (game) => (
		<Table.Td className={styles.nameCell}>
			<Tooltip
				disabled={!game.hasOutdatedMod}
				label="One of the mods installed in this game is outdated."
				position="bottom"
			>
				<span>
					<ItemName label={game.discriminator}>
						{game.hasOutdatedMod && <OutdatedMarker />}
						{game.title.display}
					</ItemName>
				</span>
			</Tooltip>
		</Table.Td>
	),
};

const provider: TableColumnBase<ProcessedInstalledGame> = {
	label: "Provider",
	width: 110,
	center: true,
	hidable: true,
	getSortValue: (game) => game.provider,
	renderCell: (game) => (
		<Table.Td>
			<ProviderBadge value={game.provider} />
		</Table.Td>
	),
};

const architecture: TableColumnBase<ProcessedInstalledGame> = {
	label: "Arch",
	width: 70,
	center: true,
	hidable: true,
	getSortValue: (game) => game.executable.architecture,
	renderCell: (game) => (
		<Table.Td>
			<ArchitectureBadge value={game.executable.architecture} />
		</Table.Td>
	),
};

const scriptingBackend: TableColumnBase<ProcessedInstalledGame> = {
	label: "Backend",
	width: 90,
	center: true,
	hidable: true,
	getSortValue: (game) => game.executable.scriptingBackend,
	renderCell: (game) => (
		<Table.Td>
			<UnityBackendBadge value={game.executable.scriptingBackend} />
		</Table.Td>
	),
};

const gameTags: TableColumnBase<ProcessedInstalledGame> = {
	label: "Tags",
	width: 120,
	center: true,
	hidable: true,
	getSortValue: (game) => getGameTagsSortValue(game.ownedGame),
	renderCell: (game) => renderGameTagsCell(game.ownedGame),
};

const engine: TableColumnBase<ProcessedInstalledGame> = {
	label: "Engine",
	width: 180,
	center: true,
	hidable: true,
	sort: (dataA, dataB) =>
		sortGamesByEngine(dataA.executable.engine, dataB.executable.engine),
	renderCell: ({ executable: { engine } }) => (
		<Table.Td
			// A bit annoying that I'm defining the column width in two places (see engineColumn.width),
			// but it's to prevent this one from being squished and hiding the version number.
			// Maybe I shouldn't be using a regular table component at all for this...
			miw={170}
		>
			<EngineBadge
				maw={70}
				value={engine?.brand}
				label={engine ? (engine.version?.display ?? "-") : undefined}
			/>
		</Table.Td>
	),
};

const installedGamesColumnsMap = {
	thumbnail,
	name,
	gameTags,
	provider,
	architecture,
	scriptingBackend,
	engine,
};

export type InstalledGameColumnsId = keyof typeof installedGamesColumnsMap;

export const installedGamesColumns = columnMapToList(installedGamesColumnsMap);
