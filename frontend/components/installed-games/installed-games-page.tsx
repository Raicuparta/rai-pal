import { GameRow } from "./game-row";
import {
  Alert,
  Box,
  Button,
  Flex,
  Input,
  Stack,
  Table,
  Text,
} from "@mantine/core";
import { useSteamApps } from "@hooks/use-steam-apps";
import { useCallback, useEffect, useState } from "react";
import { MdRefresh } from "react-icons/md";
import { Game } from "@api/bindings";

function includesIgnoreCase(term: string, text: string) {
  return text.toLowerCase().includes(term.toLowerCase());
}

function includesOneOf(term: string, texts: string[]) {
  return Boolean(texts.find((text) => includesIgnoreCase(term, text)));
}

export function InstalledGamesPage() {
  const [gameMap, isLoading, refreshGameMap, error] = useSteamApps();
  const [filteredGames, setFilteredGames] = useState<Game[]>([]);

  const changeFilter = useCallback(
    (newFilter: string) => {
      if (!gameMap) return;

      const steamApps = Object.values(gameMap);
      if (!newFilter) {
        setFilteredGames(steamApps);
        return;
      }

      setFilteredGames(
        steamApps.filter((app) =>
          includesOneOf(newFilter, [
            app.name,
            app.id.toString(),
            ...Object.values(app.executables).map(
              (executable) => executable.name
            ),
          ])
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
      <Box sx={{ overflow: "auto", flex: 1 }}>
        <Table striped>
          <thead>
            <tr>
              <Box component="th" w={100} />
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
            </tr>
          </thead>
          <tbody>
            {filteredGames.map((game, index) => (
              <GameRow index={index} key={game.id} game={game} />
            ))}
          </tbody>
        </Table>
      </Box>
    </Stack>
  );
}
