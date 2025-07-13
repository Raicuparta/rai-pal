import { atom } from "jotai";
import { ProviderId } from "@api/bindings";

export const selectedGameAtom = atom<{
	providerId: ProviderId;
	gameId: string;
} | null>(null);
