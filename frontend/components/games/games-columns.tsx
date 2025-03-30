import { Box, DefaultMantineColor, Flex, Stack, Table } from "@mantine/core";
import { TableColumnBase, columnMapToList } from "@components/table/table-head";
import { ItemName } from "../item-name";
import styles from "./games.module.css";
import { EngineBrand, DbGame, GamesSortBy, ProviderId } from "@api/bindings";
import { IconCloud, IconDeviceDesktop } from "@tabler/icons-react";
import { ProviderIcon } from "@components/providers/provider-icon";
import { gameRowHeight } from "./game-row";
import { useState } from "react";

type GamesColumn = TableColumnBase<DbGame, GamesSortBy>;

type CellProps = { readonly item: DbGame };

const thumbnail: GamesColumn = {
	hidable: true,
	width: 100,
	component: function Thumbnail({ item }: CellProps) {
		const fallbackThumbnail = "images/fallback-thumbnail.png";
		const [isBroken, setIsBroken] = useState(false);
		const thumbnailUrl =
			!isBroken && item.thumbnailUrl ? item.thumbnailUrl : fallbackThumbnail;

		return (
			<Table.Td
				bg="dark"
				className={styles.thumbnailCell}
			>
				<img
					className={styles.thumbnailBackground}
					src={thumbnailUrl}
				/>
				{(item.thumbnailUrl || isBroken) && (
					<img
						src={thumbnailUrl}
						className={styles.thumbnailForeground}
						onError={() => setIsBroken(true)}
					/>
				)}
			</Table.Td>
		);
	},
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
			bg={`var(--mantine-color-${providerColors[item.providerId]}-light)`}
			opacity={item.exePath ? 1 : 0.5}
		>
			<Stack
				justify="center"
				align="center"
			>
				<ProviderIcon
					providerId={item.providerId}
					color={`var(--mantine-color-${providerColors[item.providerId]}-light-color)`}
				/>
				{item.exePath ? <IconDeviceDesktop color="white" /> : <IconCloud />}
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
				c={item.exePath ? "white" : "grey"}
				style={{
					maxHeight: gameRowHeight,
					overflow: "hidden",
				}}
			>
				<ItemName label={item.installedGame?.discriminator}>
					{item?.displayTitle}
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
		const engineColor = item.engineBrand
			? engineColors[item.engineBrand]
			: "gray";

		const detailsText =
			item.unityBackend && item.architecture
				? `${item.unityBackend} ${item.architecture}`
				: item.architecture;

		return (
			<Table.Td
				bg={
					item.engineBrand
						? `var(--mantine-color-${engineColor}-light)`
						: "dark.4"
				}
				className={styles.engineWrapper}
				p={0}
			>
				{item.engineBrand && (
					<Box
						c={`var(--mantine-color-${engineColor}-light-color)`}
						className={styles.engineBrand}
					>
						{item.engineBrand}
					</Box>
				)}
				{!engine && <div>-</div>}
				{item.engineVersion && (
					<Box className={styles.engineVersion}>{item.engineVersion}</Box>
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
