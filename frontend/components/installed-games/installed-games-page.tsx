import { Group, Stack } from "@mantine/core";
import { useMemo, useState } from "react";
import { filterGame, includesOneOf } from "../../util/filter";
import { InstalledGameModal } from "./installed-game-modal";
import { useFilteredList } from "@hooks/use-filtered-list";
import { FilterMenu } from "@components/filters/filter-menu";
import { VirtualizedTable } from "@components/table/virtualized-table";
import { RefreshButton } from "@components/refresh-button";
import { SearchInput } from "@components/search-input";
import {
	InstalledGameColumnsId,
	installedGamesColumns,
} from "./installed-games-columns";
import { usePersistedState } from "@hooks/use-persisted-state";
import { AddGame } from "./add-game-button";
import {
	ProcessedInstalledGame,
	useProcessedInstalledGames,
} from "@hooks/use-processed-installed-games";
import { FilterSelect } from "@components/filters/filter-select";

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

const defaultColumns: InstalledGameColumnsId[] = [
	"thumbnail",
	"engine",
	"provider",
];

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
	>(defaultColumns, "installed-visible-columns");

	const filteredColumns = useMemo(
		() =>
			installedGamesColumns.filter(
				(column) => !column.hidable || visibleColumnIds.includes(column.id),
			),
		[visibleColumnIds],
	);

	const [
		filteredGames,
		sort,
		setSort,
		hiddenValues,
		setHiddenValues,
		search,
		setSearch,
	] = useFilteredList(
		"installed-games-hidden",
		filteredColumns,
		installedGames,
		filterInstalledGame,
		defaultFilter,
	);

	const isFilterActive =
		Object.values(hiddenValues).filter((filterValue) => filterValue.length > 0)
			.length > 0;

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
					setFilter={setHiddenValues}
					active={isFilterActive}
				>
					{installedGamesColumns.map((column) => (
						<FilterSelect
							key={column.id}
							column={column}
							visibleColumns={visibleColumnIds}
							onChangeVisibleColumns={setVisibleColumnIds}
							hiddenValues={hiddenValues[column.id]}
							onChange={(selectedValues) =>
								setHiddenValues({ [column.id]: selectedValues })
							}
						/>
					))}
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
