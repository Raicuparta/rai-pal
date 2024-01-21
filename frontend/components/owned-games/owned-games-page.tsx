import { Group, Stack } from "@mantine/core";
import { useMemo, useState } from "react";
import { filterGame, includesOneOf } from "../../util/filter";
import { OwnedGameModal } from "./owned-game-modal";
import { useFilteredList } from "@hooks/use-filtered-list";
import { FilterMenu } from "@components/filters/filter-menu";
import { VirtualizedTable } from "@components/table/virtualized-table";
import { RefreshButton } from "@components/refresh-button";
import { SearchInput } from "@components/search-input";
import { OwnedGameColumnsId, ownedGamesColumns } from "./owned-games-columns";
import { usePersistedState } from "@hooks/use-persisted-state";
import { FixOwnedGamesButton } from "./fix-owned-games-button";
import {
	ProcessedOwnedGame,
	useProcessedOwnedGames,
} from "@hooks/use-processed-owned-games";
import { FilterSelect } from "@components/filters/filter-select";

const defaultFilter: Record<string, (string | null)[]> = {};

const defaultColumns: OwnedGameColumnsId[] = [
	"thumbnail",
	"engine",
	"provider",
];

function filterOwnedGame(
	game: ProcessedOwnedGame,
	filter: Record<string, (string | null)[]>,
	search: string,
) {
	return (
		includesOneOf(search, [game.name]) &&
		filterGame(game, filter, ownedGamesColumns)
	);
}

export function OwnedGamesPage() {
	const ownedGames = useProcessedOwnedGames();

	const [selectedGameId, setSelectedGameId] = useState<string>();

	const selectedGame = useMemo(
		() =>
			ownedGames && selectedGameId ? ownedGames[selectedGameId] : undefined,
		[ownedGames, selectedGameId],
	);

	const [visibleColumnIds, setVisibleColumnIds] = usePersistedState<
		OwnedGameColumnsId[]
	>(defaultColumns, "owned-visible-columns");

	const filteredColumns = useMemo(
		() =>
			ownedGamesColumns.filter(
				(column) => !column.hidable || visibleColumnIds.includes(column.id),
			),
		[visibleColumnIds],
	);

	const [filteredGames, sort, setSort, filter, setFilter, search, setSearch] =
		useFilteredList(
			"owned-games-filter",
			filteredColumns,
			ownedGames,
			filterOwnedGame,
			defaultFilter,
		);

	const isFilterActive = Object.values(filter).filter(Boolean).length > 0;

	return (
		<Stack h="100%">
			{selectedGame ? (
				<OwnedGameModal
					onClose={() => setSelectedGameId(undefined)}
					game={selectedGame}
				/>
			) : null}
			<Group>
				<FixOwnedGamesButton />
				<SearchInput
					onChange={setSearch}
					value={search}
					count={filteredGames.length}
				/>
				<FilterMenu
					setFilter={setFilter}
					active={isFilterActive}
				>
					{ownedGamesColumns.map(
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
							),
					)}
				</FilterMenu>
				<RefreshButton />
			</Group>
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
