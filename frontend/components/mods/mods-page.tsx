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
import { Mod, openModFolder } from "@api/bindings";
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
            <Box component="th">Mod</Box>
            <Box component="th" w={50} />
          </tr>
        </thead>
        <tbody>
          {modLoaders.map((modLoader) => (
            <Fragment key={modLoader.id}>
              {modLoader.mods.map((mod, modIndex) => (
                <tr key={mod.path}>
                  {modIndex === 0 && (
                    <td rowSpan={modLoader.modCount}>{modLoader.id}</td>
                  )}
                  <td>
                    <Text>
                      <strong>{mod.name}</strong>{" "}
                      <code>({mod.scriptingBackend})</code>
                    </Text>
                  </td>
                  <td>
                    <Flex gap="md">
                      <ActionIcon
                        onClick={() => openModFolder(modLoader.id, mod.id)}
                        variant="default"
                        size="xl"
                      >
                        <MdFolder />
                      </ActionIcon>
                    </Flex>
                  </td>
                </tr>
              ))}
            </Fragment>
          ))}
        </tbody>
      </Table>
    </Stack>
  );
}
