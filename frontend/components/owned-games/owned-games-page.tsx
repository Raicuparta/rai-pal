import { Flex, Stack } from "@mantine/core";
import { OwnedGame } from "@api/bindings";
import { useMemo, useState } from "react";
import { filterGame, includesOneOf } from "../../util/filter";
import { OwnedGameModal } from "./owned-game-modal";
import { useFilteredList } from "@hooks/use-filtered-list";
import { FilterMenu } from "@components/filter-menu";
import { VirtualizedTable } from "@components/table/virtualized-table";
import { RefreshButton } from "@components/refresh-button";
import { SearchInput } from "@components/search-input";
import { ownedGamesAtom } from "@hooks/use-data";
import { useAtomValue } from "jotai";
import { ownedGamesColumns } from "./owned-games-columns";
import { ColumnsSelect } from "@components/columns-select";
import { usePersistedState } from "@hooks/use-persisted-state";
import { TypedSegmentedControl } from "@components/installed-games/typed-segmented-control";

const defaultFilter: Record<string, string> = {};

function filterOwnedGame(
	game: OwnedGame,
	filter: Record<string, string>,
	search: string,
) {
	return (
		includesOneOf(search, [game.name]) &&
		filterGame(game, filter, ownedGamesColumns)
	);
}

export function OwnedGamesPage() {
	const ownedGames = useAtomValue(ownedGamesAtom);

	const [selectedGame, setSelectedGame] = useState<OwnedGame>();

	const [hiddenColumns, setHiddenColumns] = usePersistedState<string[]>(
		"installed-hidden-columns",
		["provider"],
	);

	const filteredColumns = useMemo(
		() =>
			ownedGamesColumns.filter((column) => !hiddenColumns.includes(column.id)),
		[hiddenColumns],
	);

	const [filteredGames, sort, setSort, filter, setFilter, search, setSearch] =
		useFilteredList(
			"owned-games-filter",
			filteredColumns,
			ownedGames ?? [],
			filterOwnedGame,
			defaultFilter,
		);

	const isFilterActive = Object.values(filter).filter(Boolean).length > 0;

	return (
		<Stack h="100%">
			{selectedGame ? (
				<OwnedGameModal
					onClose={() => setSelectedGame(undefined)}
					game={selectedGame}
				/>
			) : null}
			<Flex gap="md">
				<SearchInput
					onChange={setSearch}
					value={search}
					count={filteredGames.length}
				/>
				<FilterMenu
					active={isFilterActive}
					setFilter={setFilter}
				>
					<Stack>
						<ColumnsSelect
							columns={ownedGamesColumns}
							hiddenIds={hiddenColumns}
							onChange={setHiddenColumns}
						/>
						{ownedGamesColumns.map(
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
			<VirtualizedTable
				data={filteredGames}
				columns={filteredColumns}
				onChangeSort={setSort}
				onClickItem={setSelectedGame}
				sort={sort}
			/>
		</Stack>
	);
}
