import { Game } from "./game";

export type AppMap = Record<string, SteamGame>;

export interface SteamGame extends Game {
  readonly libraryPath: string;
}
