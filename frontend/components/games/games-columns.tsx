import {
	BackgroundImage,
	DefaultMantineColor,
	Flex,
	Grid,
	GridCol,
	Paper,
	Stack,
	Table,
	Text,
	Tooltip,
} from "@mantine/core";
import { TableColumnBase, columnMapToList } from "@components/table/table-head";
import { ItemName } from "../item-name";
import { OutdatedMarker } from "@components/outdated-marker";
import styles from "../table/table.module.css";
import { EngineBrand, Game, GamesSortBy, ProviderId } from "@api/bindings";
import { IconCloud, IconDeviceDesktop } from "@tabler/icons-react";
import { GameTags } from "@components/game-tags/game-tags";
import { GameImage } from "@components/game-image";
import { ProviderIcon } from "@components/providers/provider-icon";
import { gameRowHeight } from "./game-row";

type GamesColumn = TableColumnBase<Game, GamesSortBy>;

type CellProps = { readonly item: Game };

const ThumbnailComponent = ({ item }: CellProps) => (
	<BackgroundImage
		src={item.thumbnailUrl ?? "images/fallback-thumbnail.png"}
		component={Table.Td}
		bg="dark"
		h={gameRowHeight}
		p={0}
	>
		<GameImage
			src={item.thumbnailUrl}
			h="100%"
			style={{
				backdropFilter: "blur(5px)",
			}}
		/>
	</BackgroundImage>
);

const thumbnail: GamesColumn = {
	hideInDetails: true,
	label: "Thumbnail",
	hideLabel: true,
	hidable: true,
	width: 100,
	component: ThumbnailComponent,
};

const providerColors: Record<ProviderId, DefaultMantineColor> = {
	Manual: "gray",
	Steam: "blue",
	Epic: "red",
	Gog: "violet",
	Xbox: "green",
	Itch: "pink",
	Ubisoft: "grape",
	Ea: "cyan",
} as const;

const StatusCell = ({ item }: CellProps) => (
	<Table.Td
		p={0}
		bg={`var(--mantine-color-${providerColors[item.providerId]}-light)`}
		opacity={item.installedGame ? 1 : 0.5}
	>
		<Tooltip
			label={
				item.installedGame
					? `Installed on ${item.providerId}`
					: `Owned on ${item.providerId}, not installed`
			}
		>
			<Stack
				justify="center"
				align="center"
			>
				<ProviderIcon
					providerId={item.providerId}
					color={`var(--mantine-color-${providerColors[item.providerId]}-light-color)`}
				/>
				{item.installedGame ? <IconDeviceDesktop /> : <IconCloud />}
			</Stack>
		</Tooltip>
	</Table.Td>
);

const status: GamesColumn = {
	label: "Status",
	hideLabel: true,
	hidable: true,
	width: 30,
	component: StatusCell,
};

const NameCell = ({ item }: CellProps) => (
	<Table.Td
		p={0}
		className={styles.nameCell}
	>
		<Tooltip
			disabled={!item.installedGame?.hasOutdatedMod}
			label="One of the mods installed in this game is outdated."
			position="bottom"
		>
			<Flex
				gap={3}
				align="center"
				h={gameRowHeight}
				p="xs"
				style={{
					overflow: "hidden",
					textOverflow: "ellipsis",
				}}
			>
				<ItemName label={item.installedGame?.discriminator}>
					{item.installedGame?.hasOutdatedMod && <OutdatedMarker />}
					{item?.title.display}
					<GameTags game={item} />
				</ItemName>
			</Flex>
		</Tooltip>
	</Table.Td>
);

const name: GamesColumn = {
	hideInDetails: true,
	label: "Game",
	sort: "Title",
	component: NameCell,
};

const engineColors: Record<EngineBrand, DefaultMantineColor> = {
	Unity: "blue",
	Unreal: "red",
	Godot: "violet",
	GameMaker: "teal",
} as const;

const engine: GamesColumn = {
	label: "Engine",
	sort: "Engine",
	width: 180,
	center: true,
	hidable: true,
	component: ({ item }: CellProps) => {
		// TODO somehow take into account the other remote engines too? Might also need to sort them.
		const engine =
			item.installedGame?.executable.engine ?? item.remoteGame?.engines?.[0];

		const engineColor = engine ? engineColors[engine.brand] : "gray";

		const scriptingBackend = item.installedGame?.executable.scriptingBackend;
		const architecture = item.installedGame?.executable.architecture;

		return (
			<Table.Td
			// A bit annoying that I'm defining the column width in two places (see engineColumn.width),
			// but it's to prevent this one from being squished and hiding the version number.
			// Maybe I shouldn't be using a regular table component at all for this...
			// miw={170}
			>
				<Paper
					style={{ overflow: "hidden" }}
					bg={`var(--mantine-color-${engineColor}-light)`}
					fz="xs"
				>
					<Grid
						gutter={0}
						justify="center"
						align="center"
					>
						<GridCol
							ta="center"
							span={scriptingBackend ? 8 : 12}
							c={`var(--mantine-color-${engineColor}-light-color)`}
							fw="bold"
						>
							{engine?.brand.toUpperCase()}
						</GridCol>
						{scriptingBackend && (
							<GridCol
								ta="center"
								bg="rgba(0, 0, 0, 0.3)"
								span={4}
								opacity={0.75}
							>
								{scriptingBackend.toUpperCase()}
							</GridCol>
						)}
						{engine?.version?.display && (
							<GridCol
								ta="center"
								bg="dark"
								span={12}
							>
								{engine.version.display}
								{architecture && (
									<Text
										component="span"
										size="xs"
										c="dark.3"
									>
										{" "}
										({architecture.toLowerCase()})
									</Text>
								)}
							</GridCol>
						)}
					</Grid>
				</Paper>
			</Table.Td>
		);
	},
};

const gamesColumnsMap = {
	status,
	thumbnail,
	name,
	engine,
};

export type GamesColumnId = keyof typeof gamesColumnsMap;

export const gamesColumns = columnMapToList(gamesColumnsMap);
