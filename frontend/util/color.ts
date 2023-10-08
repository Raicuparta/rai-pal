import {
	Architecture,
	GameEngineBrand,
	OperatingSystem,
	UnityScriptingBackend,
} from "@api/bindings";
import { DefaultMantineColor } from "@mantine/core";

type ColorRecord<T extends string> = Record<T, DefaultMantineColor>;

export const operatingSystemColor: ColorRecord<OperatingSystem> = {
	Linux: "yellow",
	Windows: "lime",
} as const;

export const architectureColor: ColorRecord<Architecture> = {
	X64: "blue",
	X86: "teal",
} as const;

export const scriptingBackendColor: ColorRecord<UnityScriptingBackend> = {
	Il2Cpp: "red",
	Mono: "grape",
} as const;

export const engineColor: ColorRecord<GameEngineBrand> = {
	Unity: "blue",
	Unreal: "red",
	Godot: "violet",
} as const;
