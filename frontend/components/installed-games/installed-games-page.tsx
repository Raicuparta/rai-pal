import { Flex, Stack } from "@mantine/core";
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
import { useGameMap } from "@hooks/use-game-map";
import { RefreshButton } from "@components/refresh-button";
import { SearchInput } from "@components/search-input";
import { ErrorPopover } from "@components/error-popover";
import { EngineSelect } from "@components/engine-select";

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
	{ id: "thumbnailUrl", label: "", width: 100 },
	{
		id: "name",
		label: "Game",
		width: undefined,
		getSortValue: (game) => game.name,
	},
	// {
	// 	id: "operatingSystem",
	// 	label: "OS",
	// 	width: 110,
	// 	center: true,
	// 	sortable: true,
	// },
	{
		id: "architecture",
		label: "Arch",
		width: 70,
		center: true,
		getSortValue: (game) => game.executable.architecture,
	},
	{
		id: "scriptingBackend",
		label: "Backend",
		width: 90,
		center: true,
		getSortValue: (game) => game.executable.scriptingBackend,
	},
	{
		id: "engine",
		label: "Engine",
		width: 170,
		center: true,
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
	},
];

export type TableSortMethod = (gameA: Game, gameB: Game) => number;

export function InstalledGamesPage() {
	const [gameMap, isLoading, refreshGameMap, refreshGame, error, clearError] =
		useGameMap();
	const [selectedGameId, setSelectedGameId] = useState<string>();

	const games = useMemo(() => Object.values(gameMap), [gameMap]);

	const [filteredGames, sort, setSort, filter, setFilter] = useFilteredList(
		tableHeaders,
		games,
		filterGame,
		defaultFilter,
	);

	const selectedGame = useMemo(
		() => (selectedGameId ? gameMap[selectedGameId] : undefined),
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
				<ErrorPopover
					error={error}
					clearError={clearError}
				>
					<RefreshButton
						loading={isLoading}
						onClick={refreshGameMap}
					/>
				</ErrorPopover>
			</Flex>
			{selectedGame ? (
				<InstalledGameModal
					game={selectedGame}
					onClose={() => setSelectedGameId(undefined)}
					refreshGame={refreshGame}
				/>
			) : null}
			<VirtualizedTable
				data={filteredGames}
				headerItems={tableHeaders}
				itemContent={InstalledGameRow}
				onChangeSort={setSort}
				onClickItem={(game) => setSelectedGameId(game.id)}
				sort={sort}
			/>
		</Stack>
	);
}
