import { Mod } from "../mod/mod";

export type ModLoader = {
  folderName: string;
  displayName: string;

  // mods: Readonly<Record<UnityScriptingBackend, Mod[]>> | undefined;
  mods: Readonly<Record<string, Mod[]>> | undefined;
};
