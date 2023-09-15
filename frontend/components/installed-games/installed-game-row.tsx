import { Badge, DefaultMantineColor } from "@mantine/core";
import {
  Architecture,
  Game,
  OperatingSystem,
  UnityScriptingBackend,
} from "@api/bindings";
import { GameName } from "./game-executable-name";

type ColorRecord<T extends string> = Record<T, DefaultMantineColor>;

const operatingSystemColor: ColorRecord<OperatingSystem> = {
  Linux: "yellow",
  Windows: "lime",
  Unknown: "dark",
} as const;

const architectureColor: ColorRecord<Architecture> = {
  X64: "blue",
  X86: "teal",
  Unknown: "dark",
} as const;

const scriptingBackendColor: ColorRecord<UnityScriptingBackend> = {
  Il2Cpp: "red",
  Mono: "grape",
} as const;

export function InstalledGameRow(_: number, game: Game) {
  return (
    <>
      <td>
        <GameName game={game} />
      </td>
      <td>
        <Badge color={operatingSystemColor[game.operatingSystem]}>
          {game.operatingSystem}
        </Badge>
      </td>
      <td>
        <Badge color={architectureColor[game.architecture]}>
          {game.architecture}
        </Badge>
      </td>
      <td>
        <Badge color={scriptingBackendColor[game.scriptingBackend]}>
          {game.scriptingBackend}
        </Badge>
      </td>
      <td>{game.unityVersion}</td>
    </>
  );
}
