import { ActionIcon, Badge, Box, Button, Flex, Menu } from "@mantine/core";
import {
  MdChevronRight,
  MdHandyman,
  MdMenu,
  MdMore,
  MdMoreVert,
  MdSettings,
} from "react-icons/md";
import {
  Game,
  GameExecutable,
  installMod,
  openGameFolder,
  openGameModsFolder,
  startGame,
} from "@api/bindings";
import { useModLoaders } from "@hooks/use-backend-data";
import { Fragment } from "react";

export type GameExecutableData = {
  game: Game;
  executable: GameExecutable;
};

export function GameExecutableRow(index: number, data: GameExecutableData) {
  // const [modLoaders] = useModLoaders();

  const nameSuffix =
    Object.values(data.game.executables).length <= 1
      ? ""
      : ` (${
          data.executable.steamLaunch?.description ||
          `${data.executable.name} ${
            data.executable.steamLaunch?.arguments || ""
          }`
        })`;

  return (
    <>
      <td>
        <Flex gap="md">
          <Menu>
            <Menu.Target>
              <ActionIcon variant="default">
                <MdMoreVert />
              </ActionIcon>
            </Menu.Target>
            <Menu.Dropdown>
              <Menu.Item
                onClick={() => openGameFolder(data.game.id, data.executable.id)}
              >
                Open Game Folder
              </Menu.Item>
              <Menu.Item
                onClick={() =>
                  openGameModsFolder(data.game.id, data.executable.id)
                }
              >
                Open Mods Folder
              </Menu.Item>
              <Menu.Item
                onClick={() => startGame(data.game.id, data.executable.id)}
              >
                Start Game
              </Menu.Item>
              {/* {modLoaders.map((modLoader) => (
                <Menu key={modLoader.id} trigger="hover" position="right-start">
                  <Menu.Target>
                    <Menu.Item>
                      Install {modLoader.id} mod <strong>{"›"}</strong>
                    </Menu.Item>
                  </Menu.Target>
                  <Menu.Dropdown>
                    <Menu.Item sx={{ display: "none" }} />
                    {modLoader.mods
                      .filter(
                        (mod) =>
                          mod.scriptingBackend ===
                          data.executable.scriptingBackend
                      )
                      .map((mod) => (
                        <Menu.Item
                          key={mod.name}
                          onClick={() =>
                            installMod(
                              modLoader.id,
                              mod.id,
                              data.game.id,
                              data.executable.id
                            )
                          }
                        >
                          Install {mod.name}
                        </Menu.Item>
                      ))}
                  </Menu.Dropdown>
                </Menu>
              ))} */}
            </Menu.Dropdown>
          </Menu>
        </Flex>
      </td>
      <td>
        {data.game.name}{" "}
        <Box component="code" sx={{ color: "GrayText" }}>
          {nameSuffix}
        </Box>
      </td>
      <td>
        <Badge
          color={
            data.executable.operatingSystem === "Linux" ? "yellow" : "lime"
          }
        >
          {data.executable.operatingSystem}
        </Badge>
      </td>
      <td>
        <Badge color={data.executable.architecture === "X64" ? "blue" : "teal"}>
          {data.executable.architecture}
        </Badge>
      </td>
      <td>
        <Badge
          color={
            data.executable.scriptingBackend === "Il2Cpp" ? "red" : "grape"
          }
        >
          {data.executable.scriptingBackend}
        </Badge>
      </td>
      <td>{data.executable.unityVersion}</td>
    </>
  );
}
