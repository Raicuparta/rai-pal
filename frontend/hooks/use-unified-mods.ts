import { useAtomValue } from "jotai";
import { localModsAtom, remoteModsAtom } from "./use-data";
import { useMemo } from "react";
import { CommonModData, LocalModData, RemoteModData } from "@api/bindings";

export type UnifiedMod = {
	common: CommonModData;
	local?: LocalModData;
	remote?: RemoteModData;
};

export function useUnifiedMods() {
	const localMods = useAtomValue(localModsAtom);
	const remoteMods = useAtomValue(remoteModsAtom);
	const unifiedMods = useMemo(() => {
		const modMap: Record<string, UnifiedMod> = {};
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
				Object.entries(localMod?.common ?? {}).filter(
					([, value]) => value != null,
				),
			) as CommonModData;

			// When a mod is downloaded, the database information is stored in the local manifest.
			// But there can be cases where the information isn't the same on both ends.
			// Local manifest information takes precedence, but if certain parts of the manifest are missing,
			// we'll just use the one from the database.
			// This might cause some discrepancies, but since this should mostly only happen when messing
			// with mods for dev purposes, I think it's ok.
			const common = {
				...remoteMod?.common,
				...cleanedUpLocalCommon,
			};

			modMap[key] = {
				common,
				local: localMod?.data,
				remote: remoteMod?.data,
			};
		}

		return modMap;
	}, [localMods, remoteMods]);

	return unifiedMods;
}
