import { atom } from "jotai";
import { GameProviderId } from "@api/bindings";

export const selectedGameAtom = atom<{
	providerId: GameProviderId;
	gameId: string;
} | null>(null);
