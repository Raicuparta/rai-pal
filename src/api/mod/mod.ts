import { ModLoader } from "../mod-loader/mod-loader";
import { UnityScriptingBackend } from "../engine/unity";

export type Mod = {
  name: string;
  scriptingBackend: UnityScriptingBackend;
  modLoader: ModLoader;
};
