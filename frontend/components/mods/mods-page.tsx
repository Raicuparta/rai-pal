import {
  ActionIcon,
  Box,
  Button,
  Flex,
  Stack,
  Table,
  Text,
} from "@mantine/core";
import { MdFolder, MdRefresh } from "react-icons/md";
import { Fragment } from "react";
import { useModLoaders } from "@hooks/use-backend-data";
import { openModFolder } from "@api/bindings";

export function ModsPage() {
  const [modLoaders, isLoading, refreshMods] = useModLoaders();

  return (
    <Stack h="100%">
      <Button
        disabled={isLoading}
        onClick={refreshMods}
        loading={isLoading}
        leftSection={<MdRefresh />}
      >
        {isLoading ? "Finding mods..." : "Refresh"}
      </Button>
      <Table withColumnBorders>
        <Table.Thead>
          <Table.Tr>
            <Box component="th" w={100}>
              Loader
            </Box>
            <Box component="th">Mod</Box>
            <Box component="th" w={50} />
          </Table.Tr>
        </Table.Thead>
        <Table.Tbody>
          {Object.values(modLoaders).map((modLoader) => (
            <Fragment key={modLoader.id}>
              {modLoader.mods.map((mod, modIndex) => (
                <Table.Tr key={mod.path}>
                  {modIndex === 0 && (
                    <Table.Td rowSpan={modLoader.mods.length}>
                      {modLoader.id}
                    </Table.Td>
                  )}
                  <Table.Td>
                    <Text>
                      <strong>{mod.name}</strong>{" "}
                      <code>({mod.scriptingBackend})</code>
                    </Text>
                  </Table.Td>
                  <Table.Td>
                    <Flex gap="md">
                      <ActionIcon
                        onClick={() => openModFolder(modLoader.id, mod.id)}
                        variant="default"
                        size="xl"
                      >
                        <MdFolder />
                      </ActionIcon>
                    </Flex>
                  </Table.Td>
                </Table.Tr>
              ))}
            </Fragment>
          ))}
        </Table.Tbody>
      </Table>
    </Stack>
  );
}
