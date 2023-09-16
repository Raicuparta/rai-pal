import { Button, Checkbox, Flex, Input, Stack, Text } from "@mantine/core";
import { MdRefresh } from "react-icons/md";
import { OwnedGameRow } from "./owned-game-row";
import { useOwnedUnityGames } from "@hooks/use-backend-data";
import { OwnedUnityGame } from "@api/bindings";
import { useState } from "react";
import { includesOneOf } from "../../util/filter";
import { OwnedGameModal } from "./owned-game-modal";
import { TableHeader } from "@components/table/table-head";
import { useFilteredList } from "@hooks/use-filtered-list";
import { FilterMenu } from "@components/filter-menu";
import { VirtualizedTable } from "@components/table/virtualized-table";

const tableHeaders: TableHeader<OwnedUnityGame, keyof OwnedUnityGame>[] = [
  { id: "name", label: "Game", width: undefined },
  { id: "osList", label: "Linux?", width: 100 },
  { id: "installed", label: "Installed?", width: 100 },
];

type Filter = {
  text: string;
  hideInstalled: boolean;
  linuxOnly: boolean;
};

const defaultFilter: Filter = {
  text: "",
  hideInstalled: false,
  linuxOnly: false,
};

const filterGame = (game: OwnedUnityGame, filter: Filter) =>
  includesOneOf(filter.text, [game.name, game.id.toString()]) &&
  (!filter.linuxOnly || game.osList.includes("Linux")) &&
  (!filter.hideInstalled || !game.installed);

export function OwnedGamesPage() {
  const [ownedGames, isLoading, refreshOwnedGames] = useOwnedUnityGames();
  const [selectedGame, setSelectedGame] = useState<OwnedUnityGame>();

  const [filteredGames, sort, setSort, filter, setFilter] = useFilteredList(
    tableHeaders,
    ownedGames,
    filterGame,
    defaultFilter
  );

  return (
    <Stack h="100%">
      {selectedGame ? <OwnedGameModal
          selectedGame={selectedGame}
          onClose={() => setSelectedGame(undefined)}
        /> : null}
      <Flex gap="md">
        <Input
          placeholder="Find..."
          onChange={(event) => setFilter({ text: event.target.value })}
          sx={{ flex: 1 }}
        />
        <FilterMenu>
          <Stack>
            <Checkbox
              checked={filter.hideInstalled}
              onChange={(event) =>
                setFilter({ hideInstalled: event.target.checked })
              }
              label="Hide installed games"
            />
            <Checkbox
              checked={filter.linuxOnly}
              onChange={(event) =>
                setFilter({ linuxOnly: event.target.checked })
              }
              label="Hide games without native Linux support"
            />
          </Stack>
        </FilterMenu>
        <Button
          leftIcon={<MdRefresh />}
          loading={isLoading}
          onClick={refreshOwnedGames}
          sx={{ flex: 1, maxWidth: 300 }}
        >
          Refresh
        </Button>
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
