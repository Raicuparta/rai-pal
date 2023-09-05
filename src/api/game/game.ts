import { GameExecutable } from "./game-executable";

export type Game = {
  readonly id: number;
  readonly name: string;
  readonly executables: Readonly<GameExecutable[]>;
  readonly distinctExecutables: Readonly<GameExecutable[]>;
};
