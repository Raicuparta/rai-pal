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
			const common = localMod?.common ?? remoteMod?.common;

			if (!common) continue;

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
