import { Button, Flex, Stack, Table } from "@mantine/core";
import { useMemo, useState } from "react";
import { includesOneOf } from "../../util/filter";
import { InstalledGameRow } from "./installed-game-row";
import { InstalledGameModal } from "./installed-game-modal";
import {
	Architecture,
	Game,
	GameEngineBrand,
	GameEngineVersion,
	OperatingSystem,
	UnityScriptingBackend,
} from "@api/bindings";
import {
	SegmentedControlData,
	TypedSegmentedControl,
} from "./typed-segmented-control";
import { TableHeader } from "@components/table/table-head";
import { Filter, useFilteredList } from "@hooks/use-filtered-list";
import { FilterMenu } from "@components/filter-menu";
import { VirtualizedTable } from "@components/table/virtualized-table";
import { RefreshButton } from "@components/refresh-button";
import { SearchInput } from "@components/search-input";
import { EngineSelect } from "@components/engine-select";
import { useAtomValue } from "jotai";
import { installedGamesAtom } from "@hooks/use-data";
import { GameName } from "./game-name";
import {
	ArchitectureBadge,
	EngineBadge,
	OperatingSystemBadge,
	UnityBackendBadge,
} from "@components/badges/color-coded-badge";
import { GameThumbnail } from "@components/game-thumbnail";

interface InstalledGamesFilter extends Filter {
	search: string;
	operatingSystem?: OperatingSystem;
	architecture?: Architecture;
	scriptingBackend?: UnityScriptingBackend;
	engine?: GameEngineBrand;
}

const defaultFilter: InstalledGamesFilter = {
	search: "",
};

const filterGame = (game: Game, filter: InstalledGamesFilter) =>
	includesOneOf(filter.search, [game.name]) &&
	(!filter.architecture ||
		game.executable.architecture === filter.architecture) &&
	(!filter.operatingSystem ||
		game.executable.operatingSystem === filter.operatingSystem) &&
	(!filter.scriptingBackend ||
		game.executable.scriptingBackend === filter.scriptingBackend) &&
	(!filter.engine || game.executable.engine?.brand === filter.engine);

const operatingSystemOptions: SegmentedControlData<OperatingSystem>[] = [
	{ label: "Any OS", value: "" },
	{ label: "Windows", value: "Windows" },
	{ label: "Linux", value: "Linux" },
];

const architectureOptions: SegmentedControlData<Architecture>[] = [
	{ label: "Any architecture", value: "" },
	{ label: "x64", value: "X64" },
	{ label: "x86", value: "X86" },
];

const scriptingBackendOptions: SegmentedControlData<UnityScriptingBackend>[] = [
	{ label: "Any backend", value: "" },
	{ label: "IL2CPP", value: "Il2Cpp" },
	{ label: "Mono", value: "Mono" },
];

const defaultVersion: GameEngineVersion = {
	major: 0,
	minor: 0,
	patch: 0,
	suffix: "",
	display: "",
};

const tableHeaders: TableHeader<Game>[] = [
	{
		id: "thumbnailUrl",
		label: "",
		width: 100,
		renderCell: (game) => <GameThumbnail url={game.thumbnailUrl} />,
	},
	{
		id: "name",
		label: "Game",
		width: undefined,
		getSortValue: (game) => game.name,
		renderCell: (game) => (
			<Table.Td>
				<GameName game={game} />
			</Table.Td>
		),
	},
	{
		id: "operatingSystem",
		label: "OS",
		width: 110,
		center: true,
		hidable: true,
		getSortValue: (game) => game.executable.operatingSystem,
		renderCell: (game) => (
			<Table.Td>
				<OperatingSystemBadge value={game.executable.operatingSystem} />
			</Table.Td>
		),
	},
	{
		id: "architecture",
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
	},
	{
		id: "scriptingBackend",
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
	},
	{
		id: "engine",
		label: "Engine",
		width: 170,
		center: true,
		hidable: true,
		sort: (dataA, dataB) => {
			const versionA = dataA.executable.engine?.version ?? defaultVersion;
			const versionB = dataB.executable.engine?.version ?? defaultVersion;
			const brandA = dataA.executable.engine?.brand ?? "";
			const brandB = dataB.executable.engine?.brand ?? "";

			return (
				brandA.localeCompare(brandB) ||
				versionA.major - versionB.major ||
				versionA.minor - versionB.minor ||
				versionA.patch - versionB.patch ||
				0
			);
		},
		renderCell: (game) => (
			<Table.Td>
				<Flex
					align="center"
					gap="xs"
				>
					<EngineBadge
						maw={70}
						value={game.executable.engine?.brand}
						label={game.executable.engine?.version?.display}
					/>
				</Flex>
			</Table.Td>
		),
	},
];

export type TableSortMethod = (gameA: Game, gameB: Game) => number;

export function InstalledGamesPage() {
	const gameMap = useAtomValue(installedGamesAtom);

	const [selectedGameId, setSelectedGameId] = useState<string>();
	const [hideTableHeaders, setHideTableHeaders] = useState<string[]>([
		"operatingSystem",
	]);

	const games = useMemo(
		() => (gameMap ? Object.values(gameMap) : []),
		[gameMap],
	);

	const filteredTableHeaders = useMemo(
		() =>
			tableHeaders.filter((header) => !hideTableHeaders.includes(header.id)),
		[hideTableHeaders],
	);

	const hidableHeaders = tableHeaders.filter((header) => header.hidable);

	const [filteredGames, sort, setSort, filter, setFilter] = useFilteredList(
		filteredTableHeaders,
		games,
		filterGame,
		defaultFilter,
	);

	const selectedGame = useMemo(
		() => (gameMap && selectedGameId ? gameMap[selectedGameId] : undefined),
		[gameMap, selectedGameId],
	);

	const isFilterActive = Boolean(
		filter.architecture ||
			filter.operatingSystem ||
			filter.scriptingBackend ||
			filter.engine,
	);

	return (
		<Stack h="100%">
			<Flex gap="md">
				<SearchInput
					onChange={setFilter}
					value={filter.search}
					count={filteredGames.length}
				/>
				<FilterMenu
					setFilter={setFilter}
					active={isFilterActive}
				>
					<Stack>
						<Button.Group>
							{hidableHeaders.map((header) => (
								<Button
									variant={
										hideTableHeaders.find((id) => id === header.id)
											? "default"
											: "light"
									}
									key={header.id}
									onClick={() => {
										setHideTableHeaders(
											hideTableHeaders.find((id) => id === header.id)
												? hideTableHeaders.filter((id) => id !== header.id)
												: [...hideTableHeaders, header.id],
										);
									}}
								>
									{header.label || header.id}
								</Button>
							))}
						</Button.Group>
						<TypedSegmentedControl
							data={operatingSystemOptions}
							onChange={(operatingSystem) => setFilter({ operatingSystem })}
							value={filter.operatingSystem}
						/>
						<TypedSegmentedControl
							data={architectureOptions}
							onChange={(architecture) => setFilter({ architecture })}
							value={filter.architecture}
						/>
						<TypedSegmentedControl
							data={scriptingBackendOptions}
							onChange={(scriptingBackend) => setFilter({ scriptingBackend })}
							value={filter.scriptingBackend}
						/>
						<EngineSelect
							onChange={(engine) => setFilter({ engine })}
							value={filter.engine}
						/>
					</Stack>
				</FilterMenu>
				<RefreshButton />
			</Flex>
			{selectedGame ? (
				<InstalledGameModal
					game={selectedGame}
					onClose={() => setSelectedGameId(undefined)}
				/>
			) : null}
			<VirtualizedTable
				data={filteredGames}
				headerItems={filteredTableHeaders}
				itemContent={InstalledGameRow(filteredTableHeaders)}
				onChangeSort={setSort}
				onClickItem={(game) => setSelectedGameId(game.id)}
				sort={sort}
			/>
		</Stack>
	);
}
