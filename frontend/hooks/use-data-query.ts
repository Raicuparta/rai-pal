import { GamesQuery } from "@api/bindings";
import { getLocalStorage, setLocalStorage } from "@util/local-storage";
import { atom, useAtom } from "jotai";

export const defaultQuery: GamesQuery = {
	sortBy: "Title",
	search: "",
	sortDescending: false,
	filter: {
		architectures: ["X64", "X86", null],
		engines: ["GameMaker", "Unity", "Godot", "Unreal", null],
		providers: [
			"Epic",
			"Gog",
			"Itch",
			"Steam",
			"Manual",
			"Xbox",
			// "Ea", // TODO not yet implemented in backend.
			// "Ubisoft", // TODO not yet implemented in backend.
			null,
		],
		tags: ["Demo", "VR", null],
		unityScriptingBackends: ["Il2Cpp", "Mono", null],
		installed: ["Installed", "NotInstalled", null],
	},
};

const storageKey = "games-query";
const gamesQueryAtom = atom<GamesQuery>(
	getLocalStorage(storageKey, defaultQuery),
);
const gamesQueryAtomWithPersistence = atom(
	(get) => get(gamesQueryAtom),
	(get, set, partialQuery: Partial<GamesQuery> | null) => {
		const previousQuery = get(gamesQueryAtom);
		const newQuery = partialQuery
			? { ...previousQuery, ...partialQuery }
			: defaultQuery;
		set(gamesQueryAtom, newQuery);
		setLocalStorage(storageKey, newQuery);
	},
);

export function useDataQuery() {
	const [query, setQuery] = useAtom(gamesQueryAtomWithPersistence);
	return [query, setQuery] as const;
}
