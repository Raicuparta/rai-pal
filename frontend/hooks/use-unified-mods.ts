import { atom } from "jotai";
import { useAtomValue } from "jotai";
import { localModsAtom, remoteModsAtom } from "./use-data";
import { GameMod } from "@api/bindings";

// Keys that go into the merged part of the unified mod.
// Local mod takes priority, remote as a fallback.
// So the keys here are things we use for filtering etc,
// but we shouldn't have stuff here where it's important to dinstinguish the local vs remote version,
// like the latest release version etc.
const mergeKeys = [
	"id",
	"title",
	"engine",
	"unityBackend",
	"architecture",
	"engineVersionRange",
] as const satisfies (keyof GameMod)[];

type MergedKey = (typeof mergeKeys)[number];

type MergedMod = Pick<GameMod, MergedKey>;

export type UnifiedMod = {
	local?: GameMod;
	remote?: GameMod;
	merged: MergedMod;
};

function mergeMods(
	local: GameMod | undefined,
	remote: GameMod | undefined,
): MergedMod {
	const merged: Record<string, unknown> = {};

	for (const key of mergeKeys) {
		const val = remote?.[key] ?? local?.[key] ?? null;
		merged[key] = val;
	}

	return merged as MergedMod;
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
			local,
			remote,
			merged: mergeMods(local, remote),
		};
	}

	return unifiedMods;
});

export function useUnifiedMods() {
	return useAtomValue(unifiedModsAtom);
}
