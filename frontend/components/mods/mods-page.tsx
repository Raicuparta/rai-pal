import { Box, Button, Flex, Stack, Table, Text } from "@mantine/core";
import { MdFolder, MdRefresh } from "react-icons/md";
import { Fragment } from "react";
import { useModLoaders } from "@hooks/use-backend-data";
import { Mod } from "@api/bindings";
import { open } from "@tauri-apps/api/shell";

export function ModsPage() {
  const [modLoaders, isLoading, refreshMods] = useModLoaders();

  return (
    <Stack h="100%">
      <Button
        disabled={isLoading}
        onClick={refreshMods}
        loading={isLoading}
        leftIcon={<MdRefresh />}
      >
        {isLoading ? "Finding mods..." : "Refresh"}
      </Button>
      <Table striped withColumnBorders>
        <thead>
          <tr>
            <Box component="th" w={100}>
              Loader
            </Box>
            <Box component="th" w={100}>
              Backend
            </Box>
            <Box component="th">Mod</Box>
            <Box component="th" w={100} />
          </tr>
        </thead>
        <tbody>
          {modLoaders.map((modLoader) => (
            <Fragment key={modLoader.id}>
              {Object.entries(modLoader.mods).map(
                ([backend, mods], backendIndex) => (
                  <Fragment key={backend}>
                    {mods.map((mod: Mod, modIndex: number) => (
                      <tr key={mod.path}>
                        {modIndex === 0 && backendIndex === 0 && (
                          <td rowSpan={modLoader.modCount}>{modLoader.id}</td>
                        )}
                        {modIndex === 0 && (
                          <td rowSpan={mods.length}>{backend}</td>
                        )}
                        <td>
                          <Text weight="bold">{mod.name}</Text>
                        </td>
                        <td>
                          <Flex gap="md">
                            <Button
                              onClick={() => open(mod.path)}
                              leftIcon={<MdFolder />}
                            >
                              Open Folder
                            </Button>
                          </Flex>
                        </td>
                      </tr>
                    ))}
                  </Fragment>
                )
              )}
            </Fragment>
          ))}
        </tbody>
      </Table>
    </Stack>
  );
}
