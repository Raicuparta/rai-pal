import { Flex, Stack } from "@mantine/core";
import { GameEngineBrand, OwnedGame } from "@api/bindings";
import { useMemo, useState } from "react";
import { includesOneOf } from "../../util/filter";
import { OwnedGameModal } from "./owned-game-modal";
import { useFilteredList } from "@hooks/use-filtered-list";
import { FilterMenu } from "@components/filter-menu";
import { VirtualizedTable } from "@components/table/virtualized-table";
import { SwitchButton } from "@components/switch-button";
import { RefreshButton } from "@components/refresh-button";
import { SearchInput } from "@components/search-input";
import { EngineSelect } from "@components/engine-select";
import { ownedGamesAtom } from "@hooks/use-data";
import { useAtomValue } from "jotai";
import { ownedGamesColumns } from "./owned-games-columns";
import { ColumnsSelect } from "@components/columns-select";

type Filter = {
	hideInstalled: boolean;
	linuxOnly: boolean;
	engine?: GameEngineBrand;
};

const defaultFilter: Filter = {
	hideInstalled: false,
	linuxOnly: false,
};

const filterGame = (game: OwnedGame, filter: Filter, search: string) =>
	includesOneOf(search, [game.name, game.id.toString()]) &&
	(!filter.linuxOnly || game.osList.includes("Linux")) &&
	(!filter.hideInstalled || !game.installed) &&
	(!filter.engine || game.engine === filter.engine);

export function OwnedGamesPage() {
	const ownedGames = useAtomValue(ownedGamesAtom);

	const [selectedGame, setSelectedGame] = useState<OwnedGame>();

	const [hiddenColumns, setHiddenColumns] = useState<string[]>([]);

	const filteredColumns = useMemo(
		() =>
			ownedGamesColumns.filter((column) => !hiddenColumns.includes(column.id)),
		[hiddenColumns],
	);

	const [filteredGames, sort, setSort, filter, setFilter, search, setSearch] =
		useFilteredList(
			filteredColumns,
			ownedGames ?? [],
			filterGame,
			defaultFilter,
		);

	const isFilterActive =
		filter.linuxOnly || filter.hideInstalled || Boolean(filter.engine);

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
						{/* TODO: define these in the owned games columns, like I did for installed games. */}
						<EngineSelect
							onChange={(engine) => setFilter({ engine })}
							value={filter.engine}
						/>
						<SwitchButton
							onChange={(value) => setFilter({ hideInstalled: value })}
							value={filter.hideInstalled}
						>
							Hide installed games
						</SwitchButton>
						<SwitchButton
							onChange={(value) => setFilter({ linuxOnly: value })}
							value={filter.linuxOnly}
						>
							Hide games without native Linux support
						</SwitchButton>
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
