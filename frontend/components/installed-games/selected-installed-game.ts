import { type InstalledGame } from "@api/bindings";
import { atom } from "jotai";

export const selectedInstalledGameAtom = atom<InstalledGame | null>(null);
