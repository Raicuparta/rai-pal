import { ActionIcon, Badge, Box, Flex, Menu } from "@mantine/core";
import { MdMoreVert } from "react-icons/md";
import {
  Game,
  GameExecutable,
  openGameFolder,
  openGameModsFolder,
  startGame,
} from "@api/bindings";
import { getGameExecutableNameSuffix } from "./game-name-suffix";
import { GameExecutableName } from "./game-executable-name";

export type GameExecutableData = {
  game: Game;
  executable: GameExecutable;
  installMod: (data?: GameExecutableData) => void;
};

export function GameExecutableRow(index: number, data: GameExecutableData) {
  return (
    <>
      <td>
        <GameExecutableName data={data} />
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
