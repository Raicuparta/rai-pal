import { DefaultMantineColor, Flex, Stack, Table } from "@mantine/core";
import { TableColumnBase, columnMapToList } from "@components/table/table-head";
import { ItemName } from "../item-name";
import styles from "../table/table.module.css";
import { Game, GamesSortBy, ProviderId } from "@api/bindings";
import { IconCloud, IconDeviceDesktop } from "@tabler/icons-react";
import { GameTags } from "@components/game-tags/game-tags";
import { GameImage } from "@components/game-image";
import { ProviderIcon } from "@components/providers/provider-icon";
import { gameRowHeight } from "./game-row";
import { EngineBadge } from "@components/engine-badge/engine-badge";

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
		{/* TODO: custom tooltips. Mantine tooltips bad for performance. */}
		<Stack
			justify="center"
			align="center"
		>
			<ProviderIcon
				providerId={item.id.providerId}
				color={`var(--mantine-color-${providerColors[item.id.providerId]}-light-color)`}
			/>
			{item.installedGame ? <IconDeviceDesktop color="white" /> : <IconCloud />}
		</Stack>
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

const engine: GamesColumn = {
	label: "Engine",
	sort: "Engine",
	width: 160,
	center: true,
	hidable: true,
	component: ({ item }: CellProps) => (
		<Table.Td>
			<EngineBadge game={item} />
		</Table.Td>
	),
};

const dateFormatter = Intl.DateTimeFormat("default", {
	year: "numeric",
	month: "short",
	day: "numeric",
});

const releaseDate: GamesColumn = {
	label: "📅",
	width: 60,
	sort: "ReleaseDate",
	component: ({ item }: CellProps) => {
		const date = item.releaseDate
			? new Date(Number(item.releaseDate) * 1000)
			: null;

		const formattedDate = date ? dateFormatter.format(date) : "-";

		return (
			<Table.Td
				ta="center"
				fz="xs"
				opacity={0.75}
			>
				{formattedDate}
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
