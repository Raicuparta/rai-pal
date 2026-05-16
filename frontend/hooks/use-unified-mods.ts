import { atom } from "jotai";
import { useAtomValue } from "jotai";
import { localModsAtom, remoteModsAtom } from "./use-data";
import { GameMod } from "@api/bindings";

export type UnifiedMod = {
	local?: GameMod;
	remote?: GameMod;
	merged: GameMod;
};

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

		if (!local && !remote) continue;

		// local common but without any nulls or undefined values,
		// to avoid overriding remote common values.
		const cleanedUpLocalCommon = Object.fromEntries(
			Object.entries(local ?? {}).filter(([, value]) => value != null),
		) as GameMod;

		// When a mod is downloaded, the database information is stored in the local manifest.
		// But there can be cases where the information isn't the same on both ends.
		// Local manifest information takes precedence, but if certain parts of the manifest are missing,
		// we'll just use the one from the database.
		// This might cause some discrepancies, but since this should mostly only happen when messing
		// with mods for dev purposes, I think it's ok.
		const merged = {
			...remote,
			...cleanedUpLocalCommon,
		};

		unifiedMods[key] = {
			local,
			remote,
			merged,
		};
	}

	return unifiedMods;
});

export function useUnifiedMods() {
	return useAtomValue(unifiedModsAtom);
}
