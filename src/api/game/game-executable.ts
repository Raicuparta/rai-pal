import { Architecture } from "./architecture";
import { SteamLaunchOption } from "./steam-app-info";
import { UnityScriptingBackend } from "../engine/unity";

export type GameExecutable = {
  id: string;
  name: string;
  isLegacy: boolean;
  modFilesPath: string;
  fullPath: string;
  architecture: Architecture;
  scriptingBackend: UnityScriptingBackend;
  unityVersion: string;
  isLinux: boolean;
  steamLaunch?: SteamLaunchOption;
};
