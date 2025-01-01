import { atom } from "jotai";
import { GameId } from "@api/bindings";

export const selectedGameAtom = atom<GameId | null>(null);
