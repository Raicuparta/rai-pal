import { atom } from "jotai";
import { InstalledGameId } from "./installed-games-page";

export const selectedInstalledGameAtom = atom<InstalledGameId | null>(null);
