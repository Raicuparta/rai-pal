import { Architecture } from "./architecture";
import { AppLaunch } from "./steam-app-info";
import { UnityScriptingBackend } from "../engine/unity";

export type GameExecutable = {
  id: string;
  name: string;
  appType?: string;
  osList?: string;

  // isLegacy: boolean;
  // modFilesPath: string;
  // fullPath: string;
  // architecture: Architecture;
  // scriptingBackend: UnityScriptingBackend;
  // unityVersion: string;
  // isLinux: boolean;
  // steamAppId?: string;
  // steamLaunchId?: string;
  // steamLaunch?: AppLaunch;
};
