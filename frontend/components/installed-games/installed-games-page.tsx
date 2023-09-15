import { Alert, Box, Button, Flex, Input, Stack, Table } from "@mantine/core";
import { useMemo, useState } from "react";
import { MdRefresh } from "react-icons/md";
import { useGameMap } from "@hooks/use-backend-data";
import { includesOneOf } from "../../util/filter";
import { TableComponents } from "react-virtuoso";
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

const getUnityVersionScore = (game: Game) => {
  const versionParts = game.unityVersion.split(".");
  let score = 0;
  for (let i = 0; i < versionParts.length; i++) {
    const versionPart = versionParts[i];
    if (versionPart === undefined) continue;
    let versionPartNumber = parseInt(versionPart);

    if (i === 0 && versionPartNumber >= 2017) {
      // Unity 2017 is Unity 6, 2018 is 7, etc.
      versionPartNumber -= 2011;
    }

    if (isNaN(versionPartNumber)) continue;

    score += Math.pow(versionPartNumber, versionParts.length - i);
  }

  return score;
};

const tableHeaders: TableHeader<Game, keyof Game>[] = [
  { id: "name", label: "Game", width: undefined },
  { id: "operatingSystem", label: "OS", width: 100 },
  { id: "architecture", label: "Arch", width: 100 },
  { id: "scriptingBackend", label: "Backend", width: 100 },
  {
    id: "unityVersion",
    label: "Unity",
    width: 100,
    customSort: (dataA, dataB) =>
      getUnityVersionScore(dataA) - getUnityVersionScore(dataB),
  },
];

export type TableSortMethod = (gameA: Game, gameB: Game) => number;

export function InstalledGamesPage() {
  const [gameMap, isLoading, refreshGameMap, error] = useGameMap();
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
          disabled={isLoading}
          onClick={refreshGameMap}
          loading={isLoading}
          sx={{ flex: 1, maxWidth: 300 }}
          leftIcon={<MdRefresh />}
        >
          {isLoading ? "Finding Steam games..." : "Refresh"}
        </Button>
      </Flex>
      {error && (
        <Alert color="red" sx={{ overflow: "auto", flex: 1 }}>
          <pre>{error}</pre>
        </Alert>
      )}
      {selectedGame && (
        <InstalledGameModal
          game={selectedGame}
          onClose={() => setSelectedGame(undefined)}
        />
      )}
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
