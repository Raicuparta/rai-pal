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

type Props = Readonly<{
  game: Game;
  executable: GameExecutable;
}>;

export function GameExecutableRow(props: Props) {
  const [modLoaders] = useModLoaders();

  const nameSuffix =
    Object.values(props.game.executables).length <= 1
      ? ""
      : ` (${
          props.executable.steamLaunch?.description ||
          `${props.executable.name} ${
            props.executable.steamLaunch?.arguments || ""
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
                onClick={() =>
                  openGameFolder(props.game.id, props.executable.id)
                }
              >
                Open Game Folder
              </Menu.Item>
              <Menu.Item
                onClick={() =>
                  openGameModsFolder(props.game.id, props.executable.id)
                }
              >
                Open Mods Folder
              </Menu.Item>
              <Menu.Item
                onClick={() => startGame(props.game.id, props.executable.id)}
              >
                Start Game
              </Menu.Item>
              {modLoaders.map((modLoader) => (
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
                          props.executable.scriptingBackend
                      )
                      .map((mod) => (
                        <Menu.Item
                          key={mod.name}
                          onClick={() =>
                            installMod(
                              modLoader.id,
                              mod.id,
                              props.game.id,
                              props.executable.id
                            )
                          }
                        >
                          Install {mod.name}
                        </Menu.Item>
                      ))}
                  </Menu.Dropdown>
                </Menu>
              ))}
            </Menu.Dropdown>
          </Menu>
        </Flex>
      </td>
      <td>
        {props.game.name}{" "}
        <Box component="code" sx={{ color: "GrayText" }}>
          {nameSuffix}
        </Box>
      </td>
      <td>
        <Badge
          color={
            props.executable.operatingSystem === "Linux" ? "yellow" : "lime"
          }
        >
          {props.executable.operatingSystem}
        </Badge>
      </td>
      <td>
        <Badge
          color={props.executable.architecture === "X64" ? "blue" : "teal"}
        >
          {props.executable.architecture}
        </Badge>
      </td>
      <td>
        <Badge
          color={
            props.executable.scriptingBackend === "Il2Cpp" ? "red" : "grape"
          }
        >
          {props.executable.scriptingBackend}
        </Badge>
      </td>
      <td>{props.executable.unityVersion}</td>
    </>
  );
}
