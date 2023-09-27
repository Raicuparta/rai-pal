import { ActionIcon, Button, Stack, Table, Text } from "@mantine/core";
import { MdFolder, MdRefresh } from "react-icons/md";
import { Fragment } from "react";
import { useModLoaders } from "@hooks/use-backend-data";
import { openModFolder } from "@api/bindings";
import { TableContainer } from "@components/table/table-container";

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
      <TableContainer>
        <Table withColumnBorders highlightOnHover>
          <Table.Thead>
            <Table.Tr>
              <Table.Th w={100}>Loader</Table.Th>
              <Table.Th>Mod</Table.Th>
              <Table.Th w={60} />
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
                    <Table.Td align="center">
                      <ActionIcon
                        onClick={() => openModFolder(modLoader.id, mod.id)}
                        variant="default"
                        size="lg"
                      >
                        <MdFolder />
                      </ActionIcon>
                    </Table.Td>
                  </Table.Tr>
                ))}
              </Fragment>
            ))}
          </Table.Tbody>
        </Table>
      </TableContainer>
    </Stack>
  );
}
