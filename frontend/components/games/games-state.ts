import { atom } from "jotai";
import { ProviderId } from "@api/bindings";

export const selectedGameAtom = atom<{
	gameId: string;
	providerId: ProviderId;
} | null>(null);
