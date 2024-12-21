import { Table, ThemeIcon, Tooltip } from "@mantine/core";
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
import { IconCloud, IconDeviceDesktop } from "@tabler/icons-react";

type GamesColumn = TableColumnBase<Game, GamesSortBy>;

type CellProps = { readonly item: Game };

const ThumbnailComponent = ({ item }: CellProps) => (
	<ThumbnailCell
		src={getThumbnailWithFallback(
			item.installedGame?.thumbnailUrl || item?.thumbnailUrl,
			item.providerId,
		)}
	/>
);

const thumbnail: GamesColumn = {
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

const name: GamesColumn = {
	hideInDetails: true,
	label: "Game",
	sort: "Title",
	component: NameCell,
};

const provider: GamesColumn = {
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

const architecture: GamesColumn = {
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

const scriptingBackend: GamesColumn = {
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

const gameTags: GamesColumn = {
	label: "Tags",
	sort: "Tags",
	width: 120,
	center: true,
	hidable: true,
	component: GameTagsCell,
};

const engine: GamesColumn = {
	label: "Engine",
	sort: "Engine",
	width: 180,
	center: true,
	hidable: true,
	component: ({ item }: CellProps) => {
		// TODO somehow take into account the other remote engines too? Might also need to sort them.
		const engine =
			item.installedGame?.executable?.engine ?? item.remoteGame?.engines?.[0];
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

const status: GamesColumn = {
	label: "Status",
	sort: "Installed",
	width: 90,
	center: true,
	hidable: true,
	component: ({ item }: CellProps) => (
		<Table.Td align="center">
			<Tooltip label={item.installedGame ? "Installed" : "Not installed"}>
				<ThemeIcon
					variant="light"
					color={item.installedGame ? "green" : "gray"}
				>
					{item.installedGame ? <IconDeviceDesktop /> : <IconCloud />}
				</ThemeIcon>
			</Tooltip>
		</Table.Td>
	),
};

const gamesColumnsMap = {
	thumbnail,
	name,
	gameTags,
	provider,
	architecture,
	scriptingBackend,
	engine,
	installed: status,
};

export type GamesColumnId = keyof typeof gamesColumnsMap;

export const gamesColumns = columnMapToList(gamesColumnsMap);
