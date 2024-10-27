import { Group, Stack } from "@mantine/core";
import { useMemo, useState } from "react";
import { filterGame, includesOneOf } from "@util/filter";
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
		includesOneOf(search, [
			game.title.display,
			...game.title.normalized,
			game.providerGameId,
		]) && filterGame(game, filter, ownedGamesColumns)
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

	const [
		filteredGames,
		sort,
		setSort,
		hiddenValues,
		setHiddenValues,
		search,
		setSearch,
	] = useFilteredList(
		"owned-games-hidden",
		filteredColumns,
		ownedGames,
		filterOwnedGame,
		defaultFilter,
	);

	const isFilterActive = Object.values(hiddenValues).filter(Boolean).length > 0;

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
				{/* TODO: add subscriptions selector once we figure out the database stuff */}
				{/* <SubscriptionsSelector /> */}
				<SearchInput
					onChange={setSearch}
					value={search}
					count={filteredGames.length}
				/>
				<FilterMenu
					setFilter={setHiddenValues}
					active={isFilterActive}
				>
					{ownedGamesColumns.map((column) => (
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
			<VirtualizedTable
				data={filteredGames}
				columns={filteredColumns}
				onChangeSort={setSort}
				onClickItem={(game) => setSelectedGameId(game.globalId)}
				sort={sort}
			/>
		</Stack>
	);
}
