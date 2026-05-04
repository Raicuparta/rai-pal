import { atom, useAtomValue } from "jotai";
import { localModLoadersAtom, remoteModLoadersAtom } from "./use-data";
import {
	LocalModLoaderData,
	RemoteModLoaderData,
	ModLoaderData,
} from "@api/bindings";

export type UnifiedModLoaderData = {
	common: ModLoaderData;
	local?: LocalModLoaderData;
	remote?: RemoteModLoaderData;
};

const unifiedModsAtom = atom((get) => {
	const localModLoaders = get(localModLoadersAtom);
	const remoteModLoaders = get(remoteModLoadersAtom);
	const unifiedModLoaders: Record<string, UnifiedModLoaderData> = {};
	const keys = [
		...new Set([
			...Object.keys(localModLoaders),
			...Object.keys(remoteModLoaders),
		]),
	].sort();

	for (const key of keys) {
		const localModLoader = localModLoaders[key];
		const remoteModLoader = remoteModLoaders[key];

		if (!localModLoader && !remoteModLoader) continue;

		// Use local common data if available, otherwise use remote
		const common = localModLoader?.common ?? remoteModLoader?.common;

		if (!common) continue;

		unifiedModLoaders[key] = {
			common,
			local: localModLoader?.data,
			remote: remoteModLoader?.data,
		};
	}

	return unifiedModLoaders;
});

export function useUnifiedModLoaders() {
	return useAtomValue(unifiedModsAtom);
}
