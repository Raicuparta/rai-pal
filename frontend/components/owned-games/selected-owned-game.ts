import { atom } from "jotai";
import { OwnedGameId } from "./owned-games-page";

export const selectedOwnedGameAtom = atom<OwnedGameId | null>(null);
