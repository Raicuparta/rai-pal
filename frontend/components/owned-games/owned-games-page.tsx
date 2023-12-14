import { Group, Stack } from "@mantine/core";
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
import { OwnedGameColumnsId, ownedGamesColumns } from "./owned-games-columns";
import { ColumnsSelect } from "@components/columns-select";
import { usePersistedState } from "@hooks/use-persisted-state";
import { TypedSegmentedControl } from "@components/installed-games/typed-segmented-control";
import { FixOwnedGamesButton } from "./fix-owned-games-button";

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

	const [selectedGameId, setSelectedGameId] = useState<string>();

	const selectedGame = useMemo(
		() =>
			ownedGames && selectedGameId ? ownedGames[selectedGameId] : undefined,
		[ownedGames, selectedGameId],
	);

	const [hiddenColumns, setHiddenColumns] = usePersistedState<
		OwnedGameColumnsId[]
	>(["provider", "gameMode"], "owned-visible-columns");

	const filteredColumns = useMemo(
		() =>
			ownedGamesColumns.filter((column) => !hiddenColumns.includes(column.id)),
		[hiddenColumns],
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
