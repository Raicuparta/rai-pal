import { GameExecutable } from "./game-executable";

export type Game = {
  readonly distinctExecutables: Readonly<GameExecutable[]>;
  readonly id: string;
  readonly name: string;
  readonly executables: Readonly<GameExecutable[]>;
};
