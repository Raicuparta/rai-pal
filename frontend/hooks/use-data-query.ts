import { GamesQuery } from "@api/bindings";
import { atom, useAtom } from "jotai";
import { useCallback } from "react";

const defaultQuery: GamesQuery = {
	sortBy: "Title",
	search: "",
	sortDescending: false,
	filter: {
		architectures: {
			X64: true,
			X86: true,
		},
		engines: {
			GameMaker: true,
			Unity: true,
			Godot: true,
			Unreal: true,
		},
		providers: {
			Epic: true,
			Gog: true,
			Itch: true,
			Steam: true,
			Manual: true,
			Xbox: true,
			Ea: true, // TODO needs to exist but hidden.
			Ubisoft: true, // TODO needs to exist but hidden.
		},
		tags: {
			Demo: true,
			VR: true,
		},
		unityScriptingBackends: {
			Il2Cpp: true,
			Mono: true,
		},
		installed: {
			Installed: true,
			NotInstalled: true,
		},
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

	return [query, setQueryExternal] as const;
}
