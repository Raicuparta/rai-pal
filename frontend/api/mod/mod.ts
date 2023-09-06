import { ModLoader } from "../mod-loader/mod-loader";

export type Mod = {
  name: string;
  // scriptingBackend: UnityScriptingBackend;
  scriptingBackend: string;
  modLoader: ModLoader;
};
