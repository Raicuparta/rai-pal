import { Flex, Stack } from "@mantine/core";
import { useMemo, useState } from "react";
import { includesOneOf } from "../../util/filter";
import { InstalledGameModal } from "./installed-game-modal";
import {
	Architecture,
	Game,
	GameEngineBrand,
	OperatingSystem,
	UnityScriptingBackend,
} from "@api/bindings";
import {
	SegmentedControlData,
	TypedSegmentedControl,
} from "./typed-segmented-control";
import { Filter, useFilteredList } from "@hooks/use-filtered-list";
import { FilterMenu } from "@components/filter-menu";
import { VirtualizedTable } from "@components/table/virtualized-table";
import { RefreshButton } from "@components/refresh-button";
import { SearchInput } from "@components/search-input";
import { EngineSelect } from "@components/engine-select";
import { useAtomValue } from "jotai";
import { installedGamesAtom } from "@hooks/use-data";
import { installedGamesColumns } from "./installed-games-columns";
import { ColumnsSelect } from "@components/columns-select";

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

export type TableSortMethod = (gameA: Game, gameB: Game) => number;

export function InstalledGamesPage() {
	const gameMap = useAtomValue(installedGamesAtom);

	const [selectedGameId, setSelectedGameId] = useState<string>();

	const [hiddenColumns, setHiddenColumns] = useState<string[]>([
		"operatingSystem",
	]);

	const games = useMemo(
		() => (gameMap ? Object.values(gameMap) : []),
		[gameMap],
	);

	const filteredColumns = useMemo(
		() =>
			installedGamesColumns.filter(
				(column) => !hiddenColumns.includes(column.id),
			),
		[hiddenColumns],
	);

	const [filteredGames, sort, setSort, filter, setFilter] = useFilteredList(
		filteredColumns,
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
						<ColumnsSelect
							columns={installedGamesColumns}
							hiddenIds={hiddenColumns}
							onChange={setHiddenColumns}
						/>
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
				columns={filteredColumns}
				onChangeSort={setSort}
				onClickItem={(game) => setSelectedGameId(game.id)}
				sort={sort}
			/>
		</Stack>
	);
}
