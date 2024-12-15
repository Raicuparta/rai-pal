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
import { getThumbnailWithFallback } from "@util/fallback-thumbnail";
import { renderGameTagsCell } from "@components/game-tags/game-tags";
import { Game, InstalledGameSortBy } from "@api/bindings";

type InstalledGameColumn = TableColumnBase<Game, InstalledGameSortBy>;

const thumbnail: InstalledGameColumn = {
	hideInDetails: true,
	label: "Thumbnail",
	hideLabel: true,
	hidable: true,
	width: 100,
	renderCell: (game) => (
		<ThumbnailCell
			src={getThumbnailWithFallback(
				game.installedGames[0]?.thumbnailUrl || game.ownedGame?.thumbnailUrl,
				game.providerId,
			)}
		/>
	),
};

const name: InstalledGameColumn = {
	hideInDetails: true,
	label: "Game",
	sort: "Title",
	renderCell: (game) => (
		<Table.Td className={styles.nameCell}>
			<Tooltip
				disabled={!game.installedGames[0]?.hasOutdatedMod}
				label="One of the mods installed in this game is outdated."
				position="bottom"
			>
				<span>
					<ItemName label={game.installedGames[0]?.discriminator}>
						{game.installedGames[0]?.hasOutdatedMod && <OutdatedMarker />}
						{game.ownedGame?.title.display}
					</ItemName>
				</span>
			</Tooltip>
		</Table.Td>
	),
};

const provider: InstalledGameColumn = {
	label: "Provider",
	sort: "Provider",
	width: 110,
	center: true,
	hidable: true,
	renderCell: (game) => (
		<Table.Td>
			<ProviderBadge value={game.providerId} />
		</Table.Td>
	),
};

const architecture: InstalledGameColumn = {
	label: "Arch",
	width: 70,
	center: true,
	hidable: true,
	renderCell: (game) => (
		<Table.Td>
			<ArchitectureBadge
				value={game.installedGames[0]?.executable.architecture}
			/>
		</Table.Td>
	),
};

const scriptingBackend: InstalledGameColumn = {
	label: "Backend",
	width: 90,
	center: true,
	hidable: true,
	renderCell: (game) => (
		<Table.Td>
			<UnityBackendBadge
				value={game.installedGames[0]?.executable.scriptingBackend}
			/>
		</Table.Td>
	),
};

const gameTags: InstalledGameColumn = {
	label: "Tags",
	width: 120,
	center: true,
	hidable: true,
	renderCell: (game) => renderGameTagsCell(game.ownedGame),
};

const engine: InstalledGameColumn = {
	label: "Engine",
	sort: "Engine",
	width: 180,
	center: true,
	hidable: true,
	renderCell: (game) => {
		const engine = game.installedGames[0]?.executable?.engine;
		return (
			<Table.Td
			// A bit annoying that I'm defining the column width in two places (see engineColumn.width),
			// but it's to prevent this one from being squished and hiding the version number.
			// Maybe I shouldn't be using a regular table component at all for this...
			// miw={170}
			>
				<EngineBadge
					maw={70}
					value={engine?.brand}
					label={engine ? (engine.version?.display ?? "-") : undefined}
				/>
			</Table.Td>
		);
	},
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
