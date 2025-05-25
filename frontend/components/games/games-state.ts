import { atom } from "jotai";
import { ProviderId } from "@api/bindings";

export const selectedGameAtom = atom<[ProviderId, string] | null>(null);
