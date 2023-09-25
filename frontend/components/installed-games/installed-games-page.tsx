import { Alert, Button, Flex, Input, Stack } from "@mantine/core";
import { useState } from "react";
import { MdRefresh } from "react-icons/md";
import { includesOneOf } from "../../util/filter";
import { InstalledGameRow } from "./installed-game-row";
import { InstalledGameModal } from "./installed-game-modal";
import {
  Architecture,
  Game,
  OperatingSystem,
  UnityScriptingBackend,
} from "@api/bindings";
import {
  SegmentedControlData,
  TypedSegmentedControl,
} from "./typed-segmented-control";
import { TableHeader } from "@components/table/table-head";
import { useFilteredList } from "@hooks/use-filtered-list";
import { FilterMenu } from "@components/filter-menu";
import { VirtualizedTable } from "@components/table/virtualized-table";
import { useGameMap } from "@hooks/use-game-map";

type Filter = {
  text: string;
  operatingSystem?: OperatingSystem;
  architecture?: Architecture;
  scriptingBackend?: UnityScriptingBackend;
};

const defaultFilter: Filter = {
  text: "",
};

const filterGame = (game: Game, filter: Filter) =>
  includesOneOf(filter.text, [game.name]) &&
  (!filter.architecture || game.architecture === filter.architecture) &&
  (!filter.operatingSystem ||
    game.operatingSystem === filter.operatingSystem) &&
  (!filter.scriptingBackend ||
    game.scriptingBackend === filter.scriptingBackend);

const operatingSystemOptions: SegmentedControlData<OperatingSystem>[] = [
  { label: "Any OS", value: "" },
  { label: "Windows", value: "Windows" },
  { label: "Linux", value: "Linux" },
];

const architectureOptions: SegmentedControlData<Architecture>[] = [
  { label: "Any architecture", value: "" },
  { label: "x64", value: "X64" },
  { label: "x86", value: "X86" },
];

const scriptingBackendOptions: SegmentedControlData<UnityScriptingBackend>[] = [
  { label: "Any backend", value: "" },
  { label: "IL2CPP", value: "Il2Cpp" },
  { label: "Mono", value: "Mono" },
];

const tableHeaders: TableHeader<Game, keyof Game>[] = [
  { id: "name", label: "Game", width: undefined },
  { id: "operatingSystem", label: "OS", width: 100 },
  { id: "architecture", label: "Arch", width: 100 },
  { id: "scriptingBackend", label: "Backend", width: 100 },
  {
    id: "unityVersion",
    label: "Unity",
    width: 120,
    customSort: (dataA, dataB) =>
      dataA.unityVersion.major - dataB.unityVersion.major ||
      dataA.unityVersion.minor - dataB.unityVersion.minor ||
      dataA.unityVersion.patch - dataB.unityVersion.patch ||
      0,
  },
];

export type TableSortMethod = (gameA: Game, gameB: Game) => number;

export function InstalledGamesPage() {
  const [gameMap, isLoading, refreshGameMap, refreshGame, error] = useGameMap();
  const [selectedGame, setSelectedGame] = useState<Game>();

  const [filteredGames, sort, setSort, filter, setFilter] = useFilteredList(
    tableHeaders,
    Object.values(gameMap),
    filterGame,
    defaultFilter
  );

  return (
    <Stack h="100%">
      <Flex gap="md">
        <Input
          placeholder="Find..."
          onChange={(event) => setFilter({ text: event.target.value })}
          sx={{ flex: 1 }}
        />
        <FilterMenu>
          <Stack>
            <TypedSegmentedControl
              data={operatingSystemOptions}
              value={filter.operatingSystem}
              onChange={(operatingSystem) => setFilter({ operatingSystem })}
            />
            <TypedSegmentedControl
              data={architectureOptions}
              value={filter.architecture}
              onChange={(architecture) => setFilter({ architecture })}
            />
            <TypedSegmentedControl
              data={scriptingBackendOptions}
              value={filter.scriptingBackend}
              onChange={(scriptingBackend) => setFilter({ scriptingBackend })}
            />
          </Stack>
        </FilterMenu>
        <Button
          onClick={refreshGameMap}
          loading={isLoading}
          sx={{ flex: 1, maxWidth: 300 }}
          leftIcon={<MdRefresh />}
        >
          {isLoading ? "Finding installed games..." : "Refresh"}
        </Button>
      </Flex>
      {error ? (
        <Alert color="red" sx={{ overflow: "auto", flex: 1 }}>
          <pre>{error}</pre>
        </Alert>
      ) : null}
      {selectedGame ? (
        <InstalledGameModal
          game={selectedGame}
          onClose={() => setSelectedGame(undefined)}
          refreshGame={refreshGame}
        />
      ) : null}
      <VirtualizedTable
        data={filteredGames}
        itemContent={InstalledGameRow}
        headerItems={tableHeaders}
        onChangeSort={setSort}
        sort={sort}
        onClickItem={setSelectedGame}
      />
    </Stack>
  );
}
