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
import { Game, GamesSortBy } from "@api/bindings";
import { GameTagsCell } from "@components/game-tags/game-tags";

type InstalledGameColumn = TableColumnBase<Game, GamesSortBy>;

type CellProps = { readonly item: Game };

const ThumbnailComponent = ({ item }: CellProps) => (
	<ThumbnailCell
		src={getThumbnailWithFallback(
			item.installedGame?.thumbnailUrl || item?.thumbnailUrl,
			item.providerId,
		)}
	/>
);

const thumbnail: InstalledGameColumn = {
	hideInDetails: true,
	label: "Thumbnail",
	hideLabel: true,
	hidable: true,
	width: 100,
	component: ThumbnailComponent,
};

const NameCell = ({ item }: CellProps) => (
	<Table.Td className={styles.nameCell}>
		<Tooltip
			disabled={!item.installedGame?.hasOutdatedMod}
			label="One of the mods installed in this game is outdated."
			position="bottom"
		>
			<span>
				<ItemName label={item.installedGame?.discriminator}>
					{item.installedGame?.hasOutdatedMod && <OutdatedMarker />}
					{item?.title.display}
				</ItemName>
			</span>
		</Tooltip>
	</Table.Td>
);

const name: InstalledGameColumn = {
	hideInDetails: true,
	label: "Game",
	sort: "Title",
	component: NameCell,
};

const provider: InstalledGameColumn = {
	label: "Provider",
	sort: "Provider",
	width: 110,
	center: true,
	hidable: true,
	component: ({ item }: CellProps) => (
		<Table.Td>
			<ProviderBadge value={item.providerId} />
		</Table.Td>
	),
};

const architecture: InstalledGameColumn = {
	label: "Arch",
	sort: "Architecture",
	width: 70,
	center: true,
	hidable: true,
	component: ({ item }: CellProps) => (
		<Table.Td>
			<ArchitectureBadge value={item.installedGame?.executable.architecture} />
		</Table.Td>
	),
};

const scriptingBackend: InstalledGameColumn = {
	label: "Backend",
	sort: "ScriptingBackend",
	width: 90,
	center: true,
	hidable: true,
	component: ({ item }: CellProps) => (
		<Table.Td>
			<UnityBackendBadge
				value={item.installedGame?.executable.scriptingBackend}
			/>
		</Table.Td>
	),
};

const gameTags: InstalledGameColumn = {
	label: "Tags",
	sort: "Tags",
	width: 120,
	center: true,
	hidable: true,
	component: GameTagsCell,
};

const engine: InstalledGameColumn = {
	label: "Engine",
	sort: "Engine",
	width: 180,
	center: true,
	hidable: true,
	component: ({ item }: CellProps) => {
		const engine = item.installedGame?.executable?.engine;
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
