import { Flex, Stack } from "@mantine/core";
import { OwnedGameRow } from "./owned-game-row";
import { useOwnedGames } from "@hooks/use-backend-data";
import { GameEngineBrand, OwnedGame } from "@api/bindings";
import { useState } from "react";
import { includesOneOf } from "../../util/filter";
import { OwnedGameModal } from "./owned-game-modal";
import { TableHeader } from "@components/table/table-head";
import { useFilteredList } from "@hooks/use-filtered-list";
import { FilterMenu } from "@components/filter-menu";
import { VirtualizedTable } from "@components/table/virtualized-table";
import { SwitchButton } from "@components/switch-button";
import { RefreshButton } from "@components/refresh-button";
import { FixOwnedGamesButton } from "./fix-owned-games-button";
import { SearchInput } from "@components/search-input";
import { ErrorPopover } from "@components/error-popover";
import { EngineSelect } from "@components/engine-select";

const tableHeaders: TableHeader<OwnedGame>[] = [
	{ id: "thumbnailUrl", label: "", width: 100 },
	{
		id: "name",
		label: "Game",
		width: undefined,
		getSortValue: (game) => game.name,
	},
	{
		id: "engine",
		label: "Engine",
		width: 100,
		center: true,
		getSortValue: (game) => game.engine,
	},
	// { id: "osList", label: "Linux?", width: 100, center: true, sortable: true },
	{
		id: "installed",
		label: "Installed",
		width: 60,
		center: true,
		getSortValue: (game) => game.installed,
	},
	{
		id: "releaseDate",
		label: "Release Date",
		width: 130,
		center: true,
		getSortValue: (game) => game.releaseDate,
	},
];

type Filter = {
	search: string;
	hideInstalled: boolean;
	linuxOnly: boolean;
	engine?: GameEngineBrand;
};

const defaultFilter: Filter = {
	search: "",
	hideInstalled: false,
	linuxOnly: false,
};

const filterGame = (game: OwnedGame, filter: Filter) =>
	includesOneOf(filter.search, [game.name, game.id.toString()]) &&
	(!filter.linuxOnly || game.osList.includes("Linux")) &&
	(!filter.hideInstalled || !game.installed) &&
	(!filter.engine || game.engine === filter.engine);

export function OwnedGamesPage() {
	const [ownedGames, isLoading, refreshOwnedGames, error, clearError] =
		useOwnedGames();
	const [selectedGame, setSelectedGame] = useState<OwnedGame>();

	const [filteredGames, sort, setSort, filter, setFilter] = useFilteredList(
		tableHeaders,
		ownedGames,
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
				<FixOwnedGamesButton />
				<SearchInput
					onChange={setFilter}
					value={filter.search}
					count={filteredGames.length}
				/>
				<FilterMenu
					active={isFilterActive}
					setFilter={setFilter}
				>
					<Stack>
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
				<ErrorPopover
					error={error}
					clearError={clearError}
				>
					<RefreshButton
						loading={isLoading}
						onClick={refreshOwnedGames}
					/>
				</ErrorPopover>
			</Flex>
			<VirtualizedTable
				data={filteredGames}
				headerItems={tableHeaders}
				itemContent={OwnedGameRow}
				onChangeSort={setSort}
				onClickItem={setSelectedGame}
				sort={sort}
			/>
		</Stack>
	);
}
