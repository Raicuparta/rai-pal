import { Box, DefaultMantineColor, Flex, Stack, Table } from "@mantine/core";
import { TableColumnBase, columnMapToList } from "@components/table/table-head";
import { ItemName } from "../item-name";
import styles from "./games.module.css";
import { EngineBrand, Game, GamesSortBy, ProviderId } from "@api/bindings";
import { IconCloud, IconDeviceDesktop } from "@tabler/icons-react";
import { GameImage } from "@components/game-image";
import { ProviderIcon } from "@components/providers/provider-icon";
import { gameRowHeight } from "./game-row";

type GamesColumn = TableColumnBase<Game, GamesSortBy>;

type CellProps = { readonly item: Game };

const thumbnail: GamesColumn = {
	hidable: true,
	width: 100,
	component: ({ item }: CellProps) => (
		<Table.Td
			bg="dark"
			p={0}
			pos="relative"
			style={{
				overflow: "hidden",
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
	),
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

const status: GamesColumn = {
	hidable: true,
	width: 30,
	component: ({ item }: CellProps) => (
		<Table.Td
			p="xs"
			bg={`var(--mantine-color-${providerColors[item.id.providerId]}-light)`}
			opacity={item.installedGame ? 1 : 0.5}
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
		</Table.Td>
	),
};

const name: GamesColumn = {
	localizationKey: "game",
	sort: "Title",
	component: ({ item }: CellProps) => (
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
					<div className={styles.tags}>
						{item.tags.sort().map((tag) => (
							<span
								className={styles.tag}
								key={tag}
							>
								{tag}
							</span>
						))}
					</div>
				</ItemName>
			</Flex>
		</Table.Td>
	),
};

const engineColors: Record<EngineBrand, DefaultMantineColor> = {
	Unity: "blue",
	Unreal: "red",
	Godot: "violet",
	GameMaker: "teal",
} as const;

const engine: GamesColumn = {
	localizationKey: "engine",
	sort: "Engine",
	width: 130,
	center: true,
	hidable: true,
	component: ({ item }: CellProps) => {
		const engine =
			item.installedGame?.executable.engine ?? item.remoteGame?.engine;

		const engineColor = engine ? engineColors[engine.brand] : "gray";

		const scriptingBackend = item.installedGame?.executable.scriptingBackend;

		const architecture = item.installedGame?.executable.architecture;

		const detailsText =
			scriptingBackend && architecture
				? `${scriptingBackend} ${architecture}`
				: architecture;

		return (
			<Table.Td
				bg={engine ? `var(--mantine-color-${engineColor}-light)` : "dark.4"}
				className={styles.engineWrapper}
				p={0}
			>
				{engine && (
					<Box
						c={`var(--mantine-color-${engineColor}-light-color)`}
						className={styles.engineBrand}
					>
						{engine?.brand}
					</Box>
				)}
				{!engine && <div>-</div>}
				{engine?.version?.display && (
					<Box className={styles.engineVersion}>{engine.version.display}</Box>
				)}
				{detailsText && (
					<Box className={styles.engineDetails}>{detailsText}</Box>
				)}
			</Table.Td>
		);
	},
};

const dateFormatter = Intl.DateTimeFormat("default", {
	year: "numeric",
	month: "long",
	day: "2-digit",
});

const releaseDate: GamesColumn = {
	localizationKey: "date",
	width: 80,
	center: true,
	sort: "ReleaseDate",
	component: ({ item }: CellProps) => {
		const date = item.releaseDate
			? new Date(Number(item.releaseDate) * 1000)
			: null;

		const formattedDate = date ? dateFormatter.formatToParts(date) : [];

		return (
			<Table.Td
				p={0}
				className={styles.dateCell}
			>
				{formattedDate.map(
					(part) =>
						part.type !== "literal" && (
							<div key={`${part.type}${part.value}`}>{part.value}</div>
						),
				)}
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
