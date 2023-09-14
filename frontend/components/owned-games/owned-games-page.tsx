import {
  Box,
  Button,
  Flex,
  Input,
  Modal,
  Stack,
  Table,
  Text,
} from "@mantine/core";
import { TableVirtuoso, TableProps, TableComponents } from "react-virtuoso";
import { MdRefresh } from "react-icons/md";
import { OwnedGameRow } from "./owned-game-row";
import { useOwnedUnityGames } from "@hooks/use-backend-data";
import { OwnedUnityGame } from "@api/bindings";
import { useCallback, useEffect, useMemo, useState } from "react";
import { includesOneOf } from "../../util/filter";
import { open } from "@tauri-apps/api/shell";
import { OwnedGameModal } from "./owned-game-modal";

const renderHeaders = () => (
  <Box component="tr" bg="dark">
    <Box component="th">Game</Box>
    <Box component="th" w={100}>
      Installed?
    </Box>
  </Box>
);

export function OwnedGamesPage() {
  const [ownedGames, isLoading, refreshOwnedGames] = useOwnedUnityGames();
  const [filteredGames, setFilteredGames] = useState<OwnedUnityGame[]>([]);
  const [selectedGame, setSelectedGame] = useState<OwnedUnityGame>();

  const changeFilter = useCallback(
    (newFilter: string) => {
      if (!ownedGames) return;

      const steamApps = Object.values(ownedGames);
      if (!newFilter) {
        setFilteredGames(steamApps);
        return;
      }

      setFilteredGames(
        steamApps.filter((game) =>
          includesOneOf(newFilter, [game.name, game.id.toString()])
        )
      );
    },
    [ownedGames]
  );

  useEffect(() => {
    changeFilter("");
  }, [changeFilter]);

  const tableComponents: TableComponents<OwnedUnityGame, any> = useMemo(
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
      {selectedGame && (
        <OwnedGameModal
          selectedGame={selectedGame}
          onClose={() => setSelectedGame(undefined)}
        />
      )}
      <Flex gap="md">
        <Input
          placeholder="Find..."
          onChange={(event) => changeFilter(event.target.value)}
          sx={{ flex: 1 }}
        />
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
      <Box sx={{ flex: 1 }}>
        <TableVirtuoso
          // eslint-disable-next-line react/forbid-component-props
          style={{ height: "100%" }}
          data={filteredGames}
          components={tableComponents}
          fixedHeaderContent={renderHeaders}
          totalCount={ownedGames.length}
          itemContent={OwnedGameRow}
        />
      </Box>
    </Stack>
  );
}
