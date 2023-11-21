import { Flex, Stack } from "@mantine/core";
import { useMemo, useState } from "react";
import { includesOneOf } from "../../util/filter";
import { InstalledGameModal } from "./installed-game-modal";
import { InstalledGame } from "@api/bindings";
import { TypedSegmentedControl } from "./typed-segmented-control";
import { useFilteredList } from "@hooks/use-filtered-list";
import { FilterMenu } from "@components/filter-menu";
import { VirtualizedTable } from "@components/table/virtualized-table";
import { RefreshButton } from "@components/refresh-button";
import { SearchInput } from "@components/search-input";
import { useAtomValue } from "jotai";
import { installedGamesAtom } from "@hooks/use-data";
import { installedGamesColumns } from "./installed-games-columns";
import { ColumnsSelect } from "@components/columns-select";
import { usePersistedState } from "@hooks/use-persisted-state";
import { AddGame } from "./add-game-button";

const defaultFilter: Record<string, string> = {};

const filterGame = (
	game: InstalledGame,
	filter: Record<string, string>,
	search: string,
) =>
	includesOneOf(search, [game.name]) &&
	installedGamesColumns.findIndex(
		(column) =>
			filter[column.id] &&
			column.getSortValue &&
			filter[column.id] !== column.getSortValue(game),
	) === -1;

export type TableSortMethod = (
	gameA: InstalledGame,
	gameB: InstalledGame,
) => number;

export function InstalledGamesPage() {
	const gameMap = useAtomValue(installedGamesAtom);

	const [selectedGameId, setSelectedGameId] = useState<string>();

	const [hiddenColumns, setHiddenColumns] = usePersistedState<string[]>(
		"installed-hidden-columns",
		["operatingSystem", "provider"],
	);

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

	const [filteredGames, sort, setSort, filter, setFilter, search, setSearch] =
		useFilteredList(
			"installed-games-filter",
			filteredColumns,
			games,
			filterGame,
			defaultFilter,
		);

	const selectedGame = useMemo(
		() => (gameMap && selectedGameId ? gameMap[selectedGameId] : undefined),
		[gameMap, selectedGameId],
	);

	const isFilterActive = Object.values(filter).filter(Boolean).length > 0;

	return (
		<Stack h="100%">
			<Flex gap="md">
				<AddGame />
				<SearchInput
					onChange={setSearch}
					value={search}
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

						{installedGamesColumns.map(
							(column) =>
								column.filterOptions && (
									<TypedSegmentedControl
										key={column.id}
										data={column.filterOptions}
										onChange={(value) => setFilter({ [column.id]: value })}
										value={filter[column.id]}
									/>
								),
						)}
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
				onClickItem={(game) => setSelectedGameId(game.executable.path)}
				sort={sort}
			/>
		</Stack>
	);
}
