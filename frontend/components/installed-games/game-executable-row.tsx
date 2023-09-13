import { Badge, Box, Button, Flex, Menu } from "@mantine/core";
import { MdHandyman, MdSettings } from "react-icons/md";
import {
  Game,
  GameExecutable,
  installMod,
  openGameFolder,
  openModsFolder,
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
              <Button radius="xl" size="xs">
                <MdSettings />
              </Button>
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
                  openModsFolder(props.game.id, props.executable.id)
                }
              >
                Open Mods Folder
              </Menu.Item>
              {/* <Menu.Item
                onClick={props.executable.addSteamLinuxLaunchParameters}
              >
                Add Linux Steam launch parameters
              </Menu.Item>
              <Menu.Item onClick={props.executable.start}>Start Game</Menu.Item> */}
            </Menu.Dropdown>
          </Menu>
          <Menu>
            <Menu.Target>
              <Button radius="xl" size="xs">
                <MdHandyman />
              </Button>
            </Menu.Target>
            <Menu.Dropdown>
              {modLoaders.map((modLoader) => (
                <Fragment key={modLoader.id}>
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
                        <Badge>{modLoader.id}</Badge> Install {mod.name}
                      </Menu.Item>
                    ))}
                </Fragment>
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
