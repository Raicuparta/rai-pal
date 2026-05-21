import { atom } from "jotai";
import { useAtomValue } from "jotai";
import { localModsAtom, remoteModsAtom } from "./use-data";
import { GameMod } from "@api/bindings";

// Anything where it's important to distinguish between local and remote versions should be included here.
export type UnifiedModVersion = Pick<GameMod, "latestVersion" | "hash">;

export interface UnifiedMod extends GameMod {
	local?: UnifiedModVersion;
	remote?: UnifiedModVersion;
}

const unifiedModsAtom = atom((get) => {
	const localMods = get(localModsAtom);
	const remoteMods = get(remoteModsAtom);
	const unifiedMods: Record<string, UnifiedMod> = {};
	const keys = [
		...new Set([...Object.keys(localMods), ...Object.keys(remoteMods)]),
	].sort();

	for (const key of keys) {
		const local = localMods[key];
		const remote = remoteMods[key];

		unifiedMods[key] = {
			...remote,
			...local,
			localVersion: local?.latestVersion,
			remoteVersion: local?.latestVersion,
		} as GameMod;
	}

	return unifiedMods;
});

export function useUnifiedMods() {
	return useAtomValue(unifiedModsAtom);
}
