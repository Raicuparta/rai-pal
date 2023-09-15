import {
  Alert,
  Box,
  Button,
  Card,
  Flex,
  Input,
  Popover,
  Stack,
  Table,
} from "@mantine/core";
import { useCallback, useMemo, useState } from "react";
import { MdFilterAlt, MdRefresh } from "react-icons/md";
import { useGameMap } from "@hooks/use-backend-data";
import { includesOneOf } from "../../util/filter";
import { TableComponents, TableVirtuoso } from "react-virtuoso";
import { GameExecutableData, InstalledGameRow } from "./installed-game-row";
import { InstalledGameModal } from "./installed-game-modal";
import {
  Architecture,
  GameExecutable,
  OperatingSystem,
  UnityScriptingBackend,
} from "@api/bindings";
import {
  SegmentedControlData,
  TypedSegmentedControl,
} from "./typed-segmented-control";
import { TableHead } from "@components/table/table-head";

type Filter = {
  text: string;
  operatingSystem?: OperatingSystem;
  architecture?: Architecture;
  scriptingBackend?: UnityScriptingBackend;
};

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

type TableHeaderId = keyof GameExecutable;

type TableHeader = {
  id: TableHeaderId;
  label: string;
  width?: number;
  customSort?: TableSortMethod;
};

const getUnityVersionScore = (data: GameExecutableData) => {
  const versionParts = data.executable.unityVersion.split(".");
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

const tableHeaders: TableHeader[] = [
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

type TableSort = {
  id: TableHeaderId;
  reverse: boolean;
  sortMethod?: TableSortMethod;
};

export type TableSortMethod = (
  dataA: GameExecutableData,
  dataB: GameExecutableData
) => number;

export function InstalledGamesPage() {
  const [gameMap, isLoading, refreshGameMap, error] = useGameMap();
  const [selectedGame, setSelectedGame] = useState<GameExecutableData>();
  const [sort, setSort] = useState<TableSort>({
    id: tableHeaders[0].id,
    reverse: false,
  });
  const [filter, setFilter] = useState<Filter>({
    text: "",
  });

  const updateFilter = (newFilter: Partial<Filter>) =>
    setFilter((previousFilter) => ({ ...previousFilter, ...newFilter }));

  const filteredGames = useMemo(() => {
    const gameExecutables = Object.values(gameMap).map((executable) => ({
      executable,
      installMod: setSelectedGame,
    }));

    const sortHeader = tableHeaders.find((header) => header.id === sort.id);

    return gameExecutables
      .filter(
        (data) =>
          includesOneOf(filter.text, [data.executable.name]) &&
          (!filter.architecture ||
            data.executable.architecture === filter.architecture) &&
          (!filter.operatingSystem ||
            data.executable.operatingSystem === filter.operatingSystem) &&
          (!filter.scriptingBackend ||
            data.executable.scriptingBackend === filter.scriptingBackend)
      )
      .sort((dataA, dataB) => {
        const multiplier = sort.reverse ? -1 : 1;

        if (sortHeader?.customSort) {
          return multiplier * sortHeader.customSort(dataA, dataB);
        }

        const valueA = dataA.executable[sort.id];
        const valueB = dataB.executable[sort.id];
        if (typeof valueA === "string" && typeof valueB === "string") {
          return multiplier * valueA.localeCompare(valueB);
        }

        return 0;
      });
  }, [filter, gameMap, sort]);

  const tableComponents: TableComponents<GameExecutableData, any> = useMemo(
    () => ({
      Table: (props) => (
        <Table {...props} highlightOnHover sx={{ tableLayout: "fixed" }} />
      ),
      TableRow: (props) => (
        <Box
          component="tr"
          sx={{ cursor: "pointer" }}
          onClick={() => setSelectedGame(props.item)}
          {...props}
        />
      ),
    }),
    [setSelectedGame]
  );

  const renderHeaders = useCallback(
    () => (
      <TableHead headers={tableHeaders} onChangeSort={setSort} sort={sort} />
    ),
    [sort, tableHeaders]
  );

  return (
    <Stack h="100%">
      <Flex gap="md">
        <Input
          placeholder="Find..."
          onChange={(event) => updateFilter({ text: event.target.value })}
          sx={{ flex: 1 }}
        />
        <Popover>
          <Popover.Target>
            <Button variant="default" leftIcon={<MdFilterAlt />}>
              Filter
            </Button>
          </Popover.Target>
          <Popover.Dropdown>
            <Stack>
              <TypedSegmentedControl
                data={operatingSystemOptions}
                value={filter.operatingSystem}
                onChange={(operatingSystem) =>
                  updateFilter({ operatingSystem })
                }
              />
              <TypedSegmentedControl
                data={architectureOptions}
                value={filter.architecture}
                onChange={(architecture) => updateFilter({ architecture })}
              />
              <TypedSegmentedControl
                data={scriptingBackendOptions}
                value={filter.scriptingBackend}
                onChange={(scriptingBackend) =>
                  updateFilter({ scriptingBackend })
                }
              />
            </Stack>
          </Popover.Dropdown>
        </Popover>
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
      {selectedGame && <InstalledGameModal data={selectedGame} />}
      <Card padding={0} sx={{ flex: 1 }}>
        <TableVirtuoso
          // eslint-disable-next-line react/forbid-component-props
          style={{ height: "100%" }}
          data={filteredGames}
          fixedHeaderContent={renderHeaders}
          components={tableComponents}
          totalCount={filteredGames.length}
          itemContent={InstalledGameRow}
        />
      </Card>
    </Stack>
  );
}
