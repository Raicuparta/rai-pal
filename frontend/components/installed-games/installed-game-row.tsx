import { Badge } from "@mantine/core";
import { Game, GameExecutable } from "@api/bindings";
import { GameExecutableName } from "./game-executable-name";

export type GameExecutableData = {
  game: Game;
  executable: GameExecutable;
  installMod: (data?: GameExecutableData) => void;
};

export function InstalledGameRow(index: number, data: GameExecutableData) {
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
