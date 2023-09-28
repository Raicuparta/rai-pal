import { Flex, Input, Stack, Text } from "@mantine/core";
import { OwnedGameRow } from "./owned-game-row";
import { useOwnedGames } from "@hooks/use-backend-data";
import { GameEngine, OwnedGame } from "@api/bindings";
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
import { ResetButton } from "@components/reset-button";

const operatingSystemOptions: SegmentedControlData<GameEngine>[] = [
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
  engine?: GameEngine;
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
    defaultFilter
  );

  const isFilterActive =
    filter.linuxOnly || filter.hideInstalled || Boolean(filter.engine);

  return (
    <Stack h="100%">
      {selectedGame ? (
        <OwnedGameModal
          selectedGame={selectedGame}
          onClose={() => setSelectedGame(undefined)}
        />
      ) : null}
      <Flex gap="md">
        <Input
          placeholder="Find..."
          value={filter.text}
          onChange={(event) => setFilter({ text: event.target.value })}
          style={{ flex: 1 }}
        />
        {(isFilterActive || filter.text) && (
          <ResetButton setFilter={setFilter} />
        )}
        <FilterMenu active={isFilterActive}>
          <Stack>
            <TypedSegmentedControl
              data={operatingSystemOptions}
              value={filter.engine}
              onChange={(engine) => setFilter({ engine })}
            />
            <SwitchButton
              value={filter.hideInstalled}
              onChange={(value) => setFilter({ hideInstalled: value })}
            >
              Hide installed games
            </SwitchButton>
            <SwitchButton
              value={filter.linuxOnly}
              onChange={(value) => setFilter({ linuxOnly: value })}
            >
              Hide games without native Linux support
            </SwitchButton>
          </Stack>
        </FilterMenu>
        <RefreshButton loading={isLoading} onClick={refreshOwnedGames} />
      </Flex>
      <Text>
        These are the Steam games you own (maybe?) that use the Unity engine
        (maybe??). {ownedGames.length} owned games.
      </Text>
      <VirtualizedTable
        data={filteredGames}
        itemContent={OwnedGameRow}
        headerItems={tableHeaders}
        sort={sort}
        onChangeSort={setSort}
        onClickItem={setSelectedGame}
      />
    </Stack>
  );
}
