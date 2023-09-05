import { Box, Button, Flex, Stack, Table, Text } from "@mantine/core";
import { useModLoaders } from "@hooks/use-mod-loaders";
import { MdFolder, MdRefresh } from "react-icons/md";
import { Fragment } from "react";

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
            <Fragment key={modLoader.folderName}>
              {/* {Object.entries(modLoader.getMods()).map(
                ([backend, mods], backendIndex) => (
                  <Fragment key={backend}>
                    {mods.map((mod, modIndex) => (
                      <tr key={mod.name}>
                        {modIndex === 0 && backendIndex === 0 && (
                          <td rowSpan={modLoader.getModCount()}>
                            {modLoader.folderName}
                          </td>
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
                              onClick={mod.openFolder}
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
              )} */}
            </Fragment>
          ))}
        </tbody>
      </Table>
    </Stack>
  );
}
