import {
  Box,
  Button,
  Card,
  Checkbox,
  Flex,
  Input,
  Popover,
  Stack,
  Table,
  Text,
} from "@mantine/core";
import { TableVirtuoso, TableComponents } from "react-virtuoso";
import { MdFilterAlt, MdRefresh } from "react-icons/md";
import { OwnedGameRow } from "./owned-game-row";
import { useOwnedUnityGames } from "@hooks/use-backend-data";
import { OwnedUnityGame } from "@api/bindings";
import { useCallback, useMemo, useState } from "react";
import { includesOneOf } from "../../util/filter";
import { OwnedGameModal } from "./owned-game-modal";
import { TableHead, TableHeader } from "@components/table/table-head";
import { useFilteredList } from "@hooks/use-filtered-list";

const tableHeaders: TableHeader<OwnedUnityGame, keyof OwnedUnityGame>[] = [
  { id: "name", label: "Game", width: undefined },
  { id: "osList", label: "Linux?", width: 100 },
  { id: "installed", label: "Installed?", width: 100 },
];

type Filter = {
  text: string;
  hideInstalled: boolean;
  linuxOnly: boolean;
};

const defaultFilter: Filter = {
  text: "",
  hideInstalled: false,
  linuxOnly: false,
};

const filterGame = (game: OwnedUnityGame, filter: Filter) =>
  includesOneOf(filter.text, [game.name, game.id.toString()]) &&
  (!filter.linuxOnly || game.osList.includes("Linux")) &&
  (!filter.hideInstalled || !game.installed);

export function OwnedGamesPage() {
  const [ownedGames, isLoading, refreshOwnedGames] = useOwnedUnityGames();
  const [selectedGame, setSelectedGame] = useState<OwnedUnityGame>();

  const [filteredGames, sort, setSort, filter, setFilter] = useFilteredList(
    tableHeaders,
    ownedGames,
    filterGame,
    defaultFilter
  );

  const renderHeaders = useCallback(
    () => (
      <TableHead headers={tableHeaders} sort={sort} onChangeSort={setSort} />
    ),
    [sort, setSort]
  );

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
          onChange={(event) => setFilter({ text: event.target.value })}
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
                  setFilter({ hideInstalled: event.target.checked })
                }
                label="Hide installed games"
              />
              <Checkbox
                checked={filter.linuxOnly}
                onChange={(event) =>
                  setFilter({ linuxOnly: event.target.checked })
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
      <Card padding={0} sx={{ flex: 1 }}>
        <TableVirtuoso
          // eslint-disable-next-line react/forbid-component-props
          style={{ height: "100%" }}
          data={filteredGames}
          components={tableComponents}
          fixedHeaderContent={renderHeaders}
          totalCount={ownedGames.length}
          itemContent={OwnedGameRow}
        />
      </Card>
    </Stack>
  );
}
