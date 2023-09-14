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
import { useCallback, useEffect, useState } from "react";
import { MdRefresh } from "react-icons/md";
import { Game, GameExecutable } from "@api/bindings";
import { useGameMap } from "@hooks/use-backend-data";
import { includesOneOf } from "../../util/filter";
import { TableVirtuoso } from "react-virtuoso";
import { GameExecutableData, GameExecutableRow } from "./game-executable-row";

const tableComponents = {
  Table: (props: TableProps) => <Table {...props} highlightOnHover />,
};

const renderHeaders = () => (
  <Box component="tr" bg="dark">
    <Box component="th" w={50} />
    <Box component="th">Game</Box>
    <Box component="th" w={50}>
      OS
    </Box>
    <Box component="th" w={50}>
      Arch
    </Box>
    <Box component="th" w={50}>
      Backend
    </Box>

    <Box component="th" w={150}>
      Unity
    </Box>
  </Box>
);

export function InstalledGamesPage() {
  const [gameMap, isLoading, refreshGameMap, error] = useGameMap();
  const [filteredGames, setFilteredGames] = useState<GameExecutableData[]>([]);

  const changeFilter = useCallback(
    (newFilter: string) => {
      if (!gameMap) return;
      const gameExecutables = Object.values(gameMap)
        .map((game) =>
          Object.values(game.executables).map((executable) => ({
            game,
            executable,
          }))
        )
        .flat();
      if (!newFilter) {
        setFilteredGames(gameExecutables);
        return;
      }
      setFilteredGames(
        gameExecutables.filter((data) =>
          includesOneOf(newFilter, [data.executable.name, data.game.name])
        )
      );
    },
    [gameMap]
  );

  useEffect(() => {
    changeFilter("");
  }, [changeFilter]);

  return (
    <Stack h="100%">
      <Flex gap="md">
        <Input
          placeholder="Find..."
          onChange={(event) => changeFilter(event.target.value)}
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
      <Box sx={{ flex: 1 }}>
        <TableVirtuoso
          // eslint-disable-next-line react/forbid-component-props
          style={{ height: "100%" }}
          data={filteredGames}
          components={tableComponents}
          fixedHeaderContent={renderHeaders}
          totalCount={filteredGames.length}
          itemContent={GameExecutableRow}
        />
      </Box>
    </Stack>
  );
}
