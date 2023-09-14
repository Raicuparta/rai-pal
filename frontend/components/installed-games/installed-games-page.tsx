import {
  Alert,
  Box,
  Button,
  Checkbox,
  Flex,
  Input,
  Popover,
  SegmentedControl,
  Stack,
  Table,
  TableProps,
} from "@mantine/core";
import { useCallback, useEffect, useMemo, useState } from "react";
import { MdFilterAlt, MdRefresh } from "react-icons/md";
import { useGameMap } from "@hooks/use-backend-data";
import { includesOneOf } from "../../util/filter";
import { TableComponents, TableVirtuoso } from "react-virtuoso";
import { GameExecutableData, InstalledGameRow } from "./installed-game-row";
import { InstalledGameModal } from "./installed-game-modal";
import {
  Architecture,
  OperatingSystem,
  UnityScriptingBackend,
} from "@api/bindings";
import {
  SegmentedControlData,
  TypedSegmentedControl,
} from "./typed-segmented-control";

const renderHeaders = () => (
  <Box component="tr" bg="dark">
    <Box component="th">Game</Box>
    <Box component="th" w={100}>
      OS
    </Box>
    <Box component="th" w={100}>
      Arch
    </Box>
    <Box component="th" w={100}>
      Backend
    </Box>

    <Box component="th" w={100}>
      Unity
    </Box>
  </Box>
);

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
  { label: "x32", value: "X32" },
];

const scriptingBackendOptions: SegmentedControlData<UnityScriptingBackend>[] = [
  { label: "Any backend", value: "" },
  { label: "IL2CPP", value: "Il2Cpp" },
  { label: "Mono", value: "Mono" },
];

export function InstalledGamesPage() {
  const [gameMap, isLoading, refreshGameMap, error] = useGameMap();
  const [selectedGame, setSelectedGame] = useState<GameExecutableData>();
  const [filter, setFilter] = useState<Filter>({
    text: "",
  });

  const updateFilter = (newFilter: Partial<Filter>) =>
    setFilter((previousFilter) => ({ ...previousFilter, ...newFilter }));

  const filteredGames = useMemo(() => {
    const gameExecutables = Object.values(gameMap)
      .map((game) =>
        Object.values(game.executables).map((executable) => ({
          game,
          executable,
          installMod: setSelectedGame,
        }))
      )
      .flat();

    return gameExecutables.filter(
      (data) =>
        includesOneOf(filter.text, [data.executable.name, data.game.name]) &&
        (!filter.architecture ||
          data.executable.architecture === filter.architecture) &&
        (!filter.operatingSystem ||
          data.executable.operatingSystem === filter.operatingSystem) &&
        (!filter.scriptingBackend ||
          data.executable.scriptingBackend === filter.scriptingBackend)
    );
  }, [filter, gameMap]);

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
      <Box sx={{ flex: 1 }}>
        <TableVirtuoso
          // eslint-disable-next-line react/forbid-component-props
          style={{ height: "100%" }}
          data={filteredGames}
          components={tableComponents}
          fixedHeaderContent={renderHeaders}
          totalCount={filteredGames.length}
          itemContent={InstalledGameRow}
        />
      </Box>
    </Stack>
  );
}
