import { Badge, Box, Button, Flex, Menu } from "@mantine/core";
import { GameExecutable } from "@api/game/game-executable";
import { Game } from "@api/game/game";
import { MdHandyman, MdSettings } from "react-icons/md";
import { useModLoaders } from "../../hooks/use-mod-loaders";
import { filterHasValue } from "@api/util/filter";
import { UnityScriptingBackend } from "@api/engine/unity";
import { Architecture } from "@api/game/architecture";
import { Mod } from "@api/mod/mod";

type Props = Readonly<{
  game: Game;
  executable: GameExecutable;
}>;

export function GameExecutableRow(props: Props) {
  const [modLoaders] = useModLoaders();

  // Merge mods of every loader, since we don't have a way to assign loaders per game yet.
  // const flatMods = modLoaders
  //   .map((modLoader) => modLoader.getMods()[props.executable.scriptingBackend])
  //   .filter(filterHasValue)
  //   .flat();

  const flatMods: Mod[] = [];

  // const nameSuffix =
  //   props.game.distinctExecutables.length <= 1
  //     ? ""
  //     : ` (${
  //         props.executable.steamLaunch?.description ||
  //         `${props.executable.name} ${
  //           props.executable.steamLaunch?.arguments || ""
  //         }`
  //       })`;

  const nameSuffix =
    props.game.distinctExecutables.length <= 1
      ? ""
      : `(${props.executable.name})`;

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
              {/* <Menu.Item onClick={props.executable.openGameFolder}>
                Open Game Folder
              </Menu.Item>
              <Menu.Item onClick={props.executable.openModsFolder}>
                Open Mods Folder
              </Menu.Item>
              {modLoaders.map((modLoader) => (
                <Menu.Item
                  key={modLoader.folderName}
                  onClick={() => props.executable.installModLoader(modLoader)}
                >
                  Install {modLoader.folderName}
                </Menu.Item>
              ))}
              <Menu.Item
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
              {flatMods.map((mod) => (
                <Menu.Item
                  key={mod.name}
                  // onClick={() => mod.install(props.executable)}
                >
                  Install {mod.name}
                </Menu.Item>
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
      {/* <td>
        <Badge color={props.executable.isLinux ? "yellow" : "lime"}>
          {props.executable.isLinux ? "Linux" : "Windows"}
        </Badge>
      </td>
      <td>
        <Badge
          color={props.executable.architecture === "x64" ? "blue" : "teal"}
        >
          {props.executable.architecture}
        </Badge>
      </td>
      <td>
        <Badge
          color={
            props.executable.scriptingBackend === "il2cpp" ? "red" : "grape"
          }
        >
          {props.executable.scriptingBackend}
        </Badge>
      </td>
      <td>{props.executable.unityVersion}</td> */}
    </>
  );
}
