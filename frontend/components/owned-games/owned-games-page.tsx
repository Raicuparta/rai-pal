import { Box, Button, Stack, Table, Text } from "@mantine/core";
import { TableVirtuoso, TableProps } from "react-virtuoso";
import { MdRefresh } from "react-icons/md";
import { useOwnedUnityGames } from "@hooks/use-owned-unity-games";
import { OwnedGameRow } from "./owned-game-row";

const tableComponents = {
  Table: (props: TableProps) => <Table {...props} highlightOnHover striped />,
};

const renderHeaders = () => (
  <Box component="tr" bg="dark">
    <Box component="th">Game</Box>
    <Box component="th" w={100}>
      Installed?
    </Box>
    <Box component="th" w={100} />
  </Box>
);

export function OwnedGamesPage() {
  const [ownedUnityGames, isLoading, updateOwnedUnityGames] =
    useOwnedUnityGames();

  return (
    <Stack h="100%">
      <Button
        leftIcon={<MdRefresh />}
        loading={isLoading}
        onClick={updateOwnedUnityGames}
      >
        Refresh
      </Button>
      <Text>
        These are the Steam games you own (maybe?) that use the Unity engine
        (maybe??). {ownedUnityGames.length} owned games.
      </Text>
      <Box sx={{ flex: 1 }}>
        <TableVirtuoso
          // eslint-disable-next-line react/forbid-component-props
          style={{ height: "100%" }}
          data={ownedUnityGames}
          components={tableComponents}
          fixedHeaderContent={renderHeaders}
          totalCount={ownedUnityGames.length}
          itemContent={OwnedGameRow}
        />
      </Box>
    </Stack>
  );
}
