import { Flex, Input, Stack } from "@mantine/core";
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
import {
	SegmentedControlData,
	TypedSegmentedControl,
} from "@components/installed-games/typed-segmented-control";
import { RefreshButton } from "@components/refresh-button";
import { FilterResetButton } from "@components/filter-reset-button";
import { FixOwnedGamesButton } from "./fix-owned-games-button";

const operatingSystemOptions: SegmentedControlData<GameEngineBrand>[] = [
	{ label: "Any Engine", value: "" },
	{ label: "Unity", value: "Unity" },
	{ label: "Unreal", value: "Unreal" },
	{ label: "Godot", value: "Godot" },
];

const tableHeaders: TableHeader<OwnedGame, keyof OwnedGame>[] = [
	{ id: "name", label: "Game", width: undefined },
	{ id: "engine", label: "Engine", width: 100, center: true },
	{ id: "osList", label: "Linux?", width: 100, center: true },
	{ id: "installed", label: "Installed?", width: 100, center: true },
	{ id: "releaseDate", label: "Release Date", width: 130, center: true },
];

type Filter = {
	text: string;
	hideInstalled: boolean;
	linuxOnly: boolean;
	engine?: GameEngineBrand;
};

const defaultFilter: Filter = {
	text: "",
	hideInstalled: false,
	linuxOnly: false,
};

const filterGame = (game: OwnedGame, filter: Filter) =>
	includesOneOf(filter.text, [game.name, game.id.toString()]) &&
	(!filter.linuxOnly || game.osList.includes("Linux")) &&
	(!filter.hideInstalled || !game.installed) &&
	(!filter.engine || game.engine === filter.engine);

export function OwnedGamesPage() {
	const [ownedGames, isLoading, refreshOwnedGames] = useOwnedGames();
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
					selectedGame={selectedGame}
				/>
			) : null}
			<Flex gap="md">
				<FixOwnedGamesButton />
				<Input
					onChange={(event) => setFilter({ text: event.target.value })}
					placeholder="Find..."
					style={{ flex: 1 }}
					value={filter.text}
				/>
				{isFilterActive || filter.text ? (
					<FilterResetButton setFilter={setFilter} />
				) : null}
				<FilterMenu active={isFilterActive}>
					<Stack>
						<TypedSegmentedControl
							data={operatingSystemOptions}
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
				<RefreshButton
					loading={isLoading}
					onClick={refreshOwnedGames}
				/>
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
