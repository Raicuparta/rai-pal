import { GamesQuery } from "@api/bindings";
import { atom, useAtom } from "jotai";
import { useCallback, useEffect } from "react";

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
			"Ea", // TODO needs to exist but hidden.
			"Ubisoft", // TODO needs to exist but hidden.
			null,
		],
		tags: ["Demo", "VR", null],
		unityScriptingBackends: ["Il2Cpp", "Mono", null],
		installed: ["Installed", "NotInstalled", null],
	},
};

export const gamesQueryAtom = atom<GamesQuery>(defaultQuery);

export function useDataQuery() {
	const [query, setQuery] = useAtom(gamesQueryAtom);

	const setQueryExternal = useCallback(
		(partialQuery: Partial<GamesQuery> | null) => {
			setQuery(
				partialQuery
					? (previousQuery) => {
							const newQuery: GamesQuery = {
								...previousQuery,
								...partialQuery,
							};

							return newQuery;
						}
					: defaultQuery,
			);
		},
		[setQuery],
	);

	useEffect(() => {
		setQuery(defaultQuery);
	}, [setQuery]);

	return [query, setQueryExternal] as const;
}
