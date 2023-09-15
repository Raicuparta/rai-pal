import { Badge, DefaultMantineColor } from "@mantine/core";
import {
  Architecture,
  GameExecutable,
  OperatingSystem,
  UnityScriptingBackend,
} from "@api/bindings";
import { GameExecutableName } from "./game-executable-name";

export type GameExecutableData = {
  executable: GameExecutable;
  installMod: (data?: GameExecutableData) => void;
};

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

export function InstalledGameRow(_: number, data: GameExecutableData) {
  return (
    <>
      <td>
        <GameExecutableName data={data} />
      </td>
      <td>
        <Badge color={operatingSystemColor[data.executable.operatingSystem]}>
          {data.executable.operatingSystem}
        </Badge>
      </td>
      <td>
        <Badge color={architectureColor[data.executable.architecture]}>
          {data.executable.architecture}
        </Badge>
      </td>
      <td>
        <Badge color={scriptingBackendColor[data.executable.scriptingBackend]}>
          {data.executable.scriptingBackend}
        </Badge>
      </td>
      <td>{data.executable.unityVersion}</td>
    </>
  );
}
