import {
	DefaultMantineColor,
	Flex,
	Paper,
	Stack,
	Table,
	Text,
	Tooltip,
} from "@mantine/core";
import { TableColumnBase, columnMapToList } from "@components/table/table-head";
import { ItemName } from "../item-name";
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
	<Table.Td
		bg="dark"
		p={0}
		pos="relative"
		style={{
			overflow: "hidden",
			borderRight: "2px solid var(--mantine-color-dark-7)",
		}}
	>
		<GameImage
			src={item.thumbnailUrl}
			h="100%"
			fit="fill"
			pos="absolute"
			top={0}
			left={0}
			style={{
				filter: "blur(10px)",
				zIndex: 0,
			}}
		/>
		<GameImage
			pos="absolute"
			top={0}
			left={0}
			src={item.thumbnailUrl}
			h="100%"
			style={{
				zIndex: 1,
			}}
		/>
	</Table.Td>
);

const thumbnail: GamesColumn = {
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
		p="xs"
		bg={`var(--mantine-color-${providerColors[item.id.providerId]}-light)`}
		opacity={item.installedGame ? 1 : 0.5}
		style={{
			borderRight: "2px solid var(--mantine-color-dark-7)",
		}}
	>
		<Tooltip
			label={
				item.installedGame
					? `Installed on ${item.id.providerId}`
					: `Owned on ${item.id.providerId}, not installed`
			}
		>
			<Stack
				justify="center"
				align="center"
			>
				<ProviderIcon
					providerId={item.id.providerId}
					color={`var(--mantine-color-${providerColors[item.id.providerId]}-light-color)`}
				/>
				{item.installedGame ? (
					<IconDeviceDesktop color="white" />
				) : (
					<IconCloud />
				)}
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
		<Flex
			gap={3}
			p="xs"
			fw="bold"
			c={item.installedGame ? "white" : "grey"}
			style={{
				maxHeight: gameRowHeight,
				overflow: "hidden",
			}}
		>
			<ItemName label={item.installedGame?.discriminator}>
				{item?.title.display}
				<GameTags game={item} />
			</ItemName>
		</Flex>
	</Table.Td>
);

const name: GamesColumn = {
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
	width: 160,
	center: true,
	hidable: true,
	component: ({ item }: CellProps) => {
		const engine =
			item.installedGame?.executable.engine ?? item.remoteGame?.engine;

		const engineColor = engine ? engineColors[engine.brand] : "gray";

		const scriptingBackend = item.installedGame?.executable.scriptingBackend;
		const architecture = item.installedGame?.executable.architecture;

		return (
			<Table.Td fz="xs">
				<Paper
					component={Stack}
					gap={0}
					bg={engine ? `var(--mantine-color-${engineColor}-light)` : undefined}
					style={{ overflow: "hidden" }}
				>
					<Flex flex={1}>
						<Flex
							justify="center"
							align="center"
							flex={4}
							c={`var(--mantine-color-${engineColor}-light-color)`}
							fw="bold"
						>
							{engine?.brand.toUpperCase()}
						</Flex>
						{scriptingBackend && (
							<Flex
								justify="center"
								align="center"
								// flex={2}
								px={5}
								bg="rgba(0, 0, 0, 0.2)"
							>
								<Text
									fz={8}
									opacity={0.5}
								>
									{scriptingBackend.toUpperCase()}
								</Text>
							</Flex>
						)}
					</Flex>
					{engine?.version?.display && (
						<Flex
							ta="center"
							bg="rgba(0, 0, 0, 0.5)"
							flex={1}
							justify="center"
							align="center"
							gap="xs"
							ff="monospace"
						>
							<Text fz="xs">{engine.version.display}</Text>
							{architecture && (
								<Text
									fz={10}
									opacity={0.5}
								>
									{architecture.toLowerCase()}
								</Text>
							)}
						</Flex>
					)}
				</Paper>
			</Table.Td>
		);
	},
};

const releaseDate: GamesColumn = {
	label: "Release",
	width: 75,
	sort: "ReleaseDate",
	component: ({ item }: CellProps) => {
		const date = item.releaseDate
			? new Date(Number(item.releaseDate) * 1000)
			: null;
		const parts = date
			? [
					date.toLocaleString("default", { year: "numeric" }),
					date.toLocaleString("default", { month: "short" }),
					date.toLocaleString("default", { day: "2-digit" }),
				]
			: ["-"];
		return (
			<Table.Td>
				<Stack
					gap={0}
					justify="center"
					align="center"
				>
					{parts.map((datePart, datePartIndex) => (
						<Text
							key={datePartIndex}
							fz="xs"
						>
							{datePart}
						</Text>
					))}
				</Stack>
			</Table.Td>
		);
	},
};

const gamesColumnsMap = {
	status,
	thumbnail,
	name,
	engine,
	releaseDate,
};

export type GamesColumnId = keyof typeof gamesColumnsMap;

export const gamesColumns = columnMapToList(gamesColumnsMap);
