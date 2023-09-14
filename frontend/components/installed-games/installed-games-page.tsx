import {
  Alert,
  Box,
  Button,
  Flex,
  Input,
  Stack,
  Table,
  TableProps,
} from "@mantine/core";
import { useCallback, useEffect, useMemo, useState } from "react";
import { MdRefresh } from "react-icons/md";
import { useGameMap } from "@hooks/use-backend-data";
import { includesOneOf } from "../../util/filter";
import { TableComponents, TableVirtuoso } from "react-virtuoso";
import { GameExecutableData, InstalledGameRow } from "./installed-game-row";
import { InstalledGameModal } from "./installed-game-modal";

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
};

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

    return gameExecutables.filter((data) =>
      includesOneOf(filter.text, [data.executable.name, data.game.name])
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
