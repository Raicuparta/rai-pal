import {
  Architecture,
  OperatingSystem,
  UnityScriptingBackend,
} from "@api/bindings";
import { DefaultMantineColor } from "@mantine/core";

type ColorRecord<T extends string> = Record<T, DefaultMantineColor>;

export const operatingSystemColor: ColorRecord<OperatingSystem> = {
  Linux: "yellow",
  Windows: "lime",
  Unknown: "dark",
} as const;

export const architectureColor: ColorRecord<Architecture> = {
  X64: "blue",
  X32: "teal",
  Unknown: "dark",
} as const;

export const scriptingBackendColor: ColorRecord<UnityScriptingBackend> = {
  Il2Cpp: "red",
  Mono: "grape",
} as const;
