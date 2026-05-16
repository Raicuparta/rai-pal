import { atom } from "jotai";
import { useAtomValue } from "jotai";
import { localModsAtom, remoteModsAtom } from "./use-data";
import { DatabaseEntry } from "@api/bindings";

export type UnifiedMod = {
	local?: DatabaseEntry;
	remote?: DatabaseEntry;
	merged: DatabaseEntry;
};

const unifiedModsAtom = atom((get) => {
	const localMods = get(localModsAtom);
	const remoteMods = get(remoteModsAtom);
	const unifiedMods: Record<string, UnifiedMod> = {};
	const keys = [
		...new Set([...Object.keys(localMods), ...Object.keys(remoteMods)]),
	].sort();

	for (const key of keys) {
		const localMod = localMods[key];
		const remoteMod = remoteMods[key];

		if (!localMod && !remoteMod) continue;

		// local common but without any nulls or undefined values,
		// to avoid overriding remote common values.
		const cleanedUpLocalCommon = Object.fromEntries(
			Object.entries(localMod?.manifest ?? {}).filter(
				([, value]) => value != null,
			),
		) as DatabaseEntry;

		// When a mod is downloaded, the database information is stored in the local manifest.
		// But there can be cases where the information isn't the same on both ends.
		// Local manifest information takes precedence, but if certain parts of the manifest are missing,
		// we'll just use the one from the database.
		// This might cause some discrepancies, but since this should mostly only happen when messing
		// with mods for dev purposes, I think it's ok.
		const merged = {
			...remoteMod,
			...cleanedUpLocalCommon,
		};

		unifiedMods[key] = {
			local: localMod?.manifest,
			remote: remoteMod,
			merged,
		};
	}

	return unifiedMods;
});

export function useUnifiedMods() {
	return useAtomValue(unifiedModsAtom);
}
