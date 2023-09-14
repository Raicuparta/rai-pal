import {
  Box,
  Button,
  Checkbox,
  Flex,
  Input,
  Menu,
  Popover,
  Stack,
  Table,
  Text,
} from "@mantine/core";
import { TableVirtuoso, TableComponents } from "react-virtuoso";
import { MdFilter, MdFilter1, MdFilterAlt, MdRefresh } from "react-icons/md";
import { OwnedGameRow } from "./owned-game-row";
import { useOwnedUnityGames } from "@hooks/use-backend-data";
import { OwnedUnityGame } from "@api/bindings";
import { useCallback, useEffect, useMemo, useState } from "react";
import { includesOneOf } from "../../util/filter";
import { OwnedGameModal } from "./owned-game-modal";

const renderHeaders = () => (
  <Box component="tr" bg="dark">
    <Box component="th">Game</Box>
    <Box component="th" w={120}>
      Native Linux?
    </Box>
    <Box component="th" w={100}>
      Installed?
    </Box>
  </Box>
);

type Filter = {
  text: string;
  hideInstalled: boolean;
  linuxOnly: boolean;
};

export function OwnedGamesPage() {
  const [ownedGames, isLoading, refreshOwnedGames] = useOwnedUnityGames();
  const [selectedGame, setSelectedGame] = useState<OwnedUnityGame>();
  const [filter, setFilter] = useState<Filter>({
    text: "",
    hideInstalled: false,
    linuxOnly: false,
  });

  const updateFilter = (newFilter: Partial<Filter>) =>
    setFilter((previousFilter) => ({ ...previousFilter, ...newFilter }));

  const filteredGames = useMemo(() => {
    const steamApps = Object.values(ownedGames);

    return steamApps.filter(
      (game) =>
        includesOneOf(filter.text, [game.name, game.id.toString()]) &&
        (!filter.linuxOnly || game.osList.includes("Linux")) &&
        (!filter.hideInstalled || !game.installed)
    );
  }, [filter, ownedGames]);

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
              <Checkbox
                checked={filter.hideInstalled}
                onChange={(event) =>
                  updateFilter({ hideInstalled: event.target.checked })
                }
                label="Hide installed games"
              />
              <Checkbox
                checked={filter.linuxOnly}
                onChange={(event) =>
                  updateFilter({ linuxOnly: event.target.checked })
                }
                label="Hide games without native Linux support"
              />
            </Stack>
          </Popover.Dropdown>
        </Popover>
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
