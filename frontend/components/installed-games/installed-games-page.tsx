import { Group, Stack } from "@mantine/core";
import { useMemo, useState } from "react";
import { filterGame, includesOneOf } from "../../util/filter";
import { InstalledGameModal } from "./installed-game-modal";
import { useFilteredList } from "@hooks/use-filtered-list";
import { FilterMenu } from "@components/filter-menu";
import { VirtualizedTable } from "@components/table/virtualized-table";
import { RefreshButton } from "@components/refresh-button";
import { SearchInput } from "@components/search-input";
import {
	InstalledGameColumnsId,
	installedGamesColumns,
} from "./installed-games-columns";
import { ColumnsSelect } from "@components/columns-select";
import { usePersistedState } from "@hooks/use-persisted-state";
import { AddGame } from "./add-game-button";
import {
	ProcessedInstalledGame,
	useProcessedInstalledGames,
} from "@hooks/use-processed-installed-games";
import { FilterSelect } from "@components/filter-select";
import { DebugData } from "@components/debug-data";

const defaultFilter: Record<string, (string | null)[]> = {};

function filterInstalledGame(
	game: ProcessedInstalledGame,
	filter: Record<string, (string | null)[]>,
	search: string,
) {
	return (
		includesOneOf(search, [game.name]) &&
		filterGame(game, filter, installedGamesColumns)
	);
}

export type TableSortMethod = (
	gameA: ProcessedInstalledGame,
	gameB: ProcessedInstalledGame,
) => number;

export function InstalledGamesPage() {
	const installedGames = useProcessedInstalledGames();

	const [selectedGameId, setSelectedGameId] = useState<string>();

	const selectedGame = useMemo(
		() =>
			installedGames && selectedGameId
				? installedGames[selectedGameId]
				: undefined,
		[installedGames, selectedGameId],
	);

	const [visibleColumnIds, setVisibleColumnIds] = usePersistedState<
		InstalledGameColumnsId[]
	>(["thumbnail", "engine"], "installed-visible-columns");

	const filteredColumns = useMemo(
		() =>
			installedGamesColumns.filter(
				(column) => !column.hidable || visibleColumnIds.includes(column.id),
			),
		[visibleColumnIds],
	);

	const [filteredGames, sort, setSort, filter, setFilter, search, setSearch] =
		useFilteredList(
			"installed-games-filter",
			filteredColumns,
			installedGames,
			filterInstalledGame,
			defaultFilter,
		);

	const isFilterActive = Object.values(filter).filter(Boolean).length > 0;

	return (
		<Stack h="100%">
			<Group>
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
					{installedGamesColumns.map(
						(column) =>
							column.filterOptions && (
								<FilterSelect
									key={column.id}
									column={column}
									visibleColumns={visibleColumnIds}
									onChangeVisibleColumns={setVisibleColumnIds}
									hiddenValues={filter[column.id]}
									onChange={(selectedValues) =>
										setFilter({ [column.id]: selectedValues })
									}
								/>
								// <TypedSegmentedControl
								// 	key={column.id}
								// 	data={column.filterOptions}
								// 	onChange={(value) => setFilter({ [column.id]: value })}
								// 	unavailableValues={column.unavailableValues}
								// 	value={filter[column.id]}
								// />
							),
					)}
					<DebugData data={filter} />
				</FilterMenu>
				<RefreshButton />
			</Group>
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
