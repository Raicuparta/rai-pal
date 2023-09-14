import { ActionIcon, Badge, Box, Flex, Menu } from "@mantine/core";
import { MdMoreVert } from "react-icons/md";
import {
  Game,
  GameExecutable,
  openGameFolder,
  openGameModsFolder,
  startGame,
} from "@api/bindings";

export type GameExecutableData = {
  game: Game;
  executable: GameExecutable;
  installMod: (data?: GameExecutableData) => void;
};

export function GameExecutableRow(index: number, data: GameExecutableData) {
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
              <Menu.Item onClick={() => data.installMod(data)}>
                Install Mod...
              </Menu.Item>
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
