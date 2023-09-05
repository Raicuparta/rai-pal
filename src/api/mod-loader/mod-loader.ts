import { Mod } from "../mod/mod";
import { UnityScriptingBackend } from "../engine/unity";

export type ModLoader = {
  folderName: string;
  displayName: string;

  mods: Readonly<Record<UnityScriptingBackend, Mod[]>> | undefined;
};
