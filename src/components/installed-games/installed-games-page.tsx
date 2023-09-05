import { GameRow } from "./game-row";
import { Box, Button, Flex, Input, Stack, Table } from "@mantine/core";
import { useSteamApps } from "@hooks/use-steam-apps";
import { useCallback, useEffect, useState } from "react";
import { SteamGame } from "@api/game/steam-game";
import { MdRefresh } from "react-icons/md";

function includesIgnoreCase(term: string, text: string) {
  return text.toLowerCase().includes(term.toLowerCase());
}

function includesOneOf(term: string, texts: string[]) {
  return Boolean(texts.find((text) => includesIgnoreCase(term, text)));
}

export function InstalledGamesPage() {
  const [steamAppMap, isLoading, refreshSteamApps] = useSteamApps();
  const [filteredSteamApps, setFilteredSteamApps] = useState<SteamGame[]>([]);

  const changeFilter = useCallback(
    (newFilter: string) => {
      if (!steamAppMap) return;

      const steamApps = Object.values(steamAppMap);
      if (!newFilter) {
        setFilteredSteamApps(steamApps);
        return;
      }

      setFilteredSteamApps(
        steamApps.filter((app) =>
          includesOneOf(newFilter, [
            app.name,
            app.id,
            ...app.distinctExecutables.map((executable) => executable.name),
          ])
        )
      );
    },
    [steamAppMap]
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
          onClick={refreshSteamApps}
          loading={isLoading}
          sx={{ flex: 1, maxWidth: 300 }}
          leftIcon={<MdRefresh />}
        >
          {isLoading ? "Finding Steam games..." : "Refresh"}
        </Button>
      </Flex>
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
            {filteredSteamApps.map((steamApp, index) => (
              <GameRow index={index} key={steamApp.id} steamApp={steamApp} />
            ))}
          </tbody>
        </Table>
      </Box>
    </Stack>
  );
}
