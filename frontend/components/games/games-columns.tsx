import { DefaultMantineColor, Flex, Stack } from "@mantine/core";
import { TableColumnBase, columnMapToList } from "@components/table/table-head";
import { ItemName } from "../item-name";
import { Game, GamesSortBy, ProviderId } from "@api/bindings";
import { IconCloud, IconDeviceDesktop } from "@tabler/icons-react";
import { GameTags } from "@components/game-tags/game-tags";
import { GameImage } from "@components/game-image";
import { ProviderIcon } from "@components/providers/provider-icon";
import { gameRowHeight } from "./game-row";
import { EngineBadge } from "@components/engine-badge/engine-badge";
import { css } from "@styled-system/css";

type GamesColumn = TableColumnBase<Game, GamesSortBy>;

type CellProps = { readonly item: Game };

const ThumbnailComponent = ({ item }: CellProps) => (
	<td
		className={css({
			overflow: "hidden",
			borderRight: "2px solid dark.800",
			backgroundColor: "dark.800",
			position: "relative",
		})}
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
	</td>
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
	<td
		style={{
			backgroundColor: `var(--mantine-color-${providerColors[item.id.providerId]}-light)`, // TODO
		}}
		className={css({
			padding: 1,
			opacity: item.installedGame ? 1 : 0.5,
			borderRight: "2px solid",
			borderColor: "dark.800",
			fontSize: "xs",
		})}
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
	</td>
);

const status: GamesColumn = {
	label: "Status",
	hideLabel: true,
	hidable: true,
	width: 30,
	component: StatusCell,
};

const NameCell = ({ item }: CellProps) => (
	<td
		className={css({
			minWidth: "10em",
			wordBreak: "break-word",
			padding: 1,
		})}
	>
		<Flex
			gap={3}
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
	</td>
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
		<td>
			<EngineBadge game={item} />
		</td>
	),
};

const dateFormatter = Intl.DateTimeFormat("default", {
	year: "numeric",
	month: "short",
	day: "numeric",
});

const releaseDate: GamesColumn = {
	label: "📅",
	width: 50,
	sort: "ReleaseDate",
	component: ({ item }: CellProps) => {
		const date = item.releaseDate
			? new Date(Number(item.releaseDate) * 1000)
			: null;

		const formattedDate = date ? dateFormatter.format(date) : "-";

		return (
			<td
				className={css({
					textAlign: "center",
					fontSize: "xs",
					padding: 1,
					opacity: 0.75,
				})}
			>
				{formattedDate}
			</td>
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
