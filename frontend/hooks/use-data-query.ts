import { GamesQuery } from "@api/bindings";
import { useAppSettingSingle } from "./use-app-setting-single";
import { useEffect, useRef } from "react";

export const defaultQuery: GamesQuery = {
	sortBy: "Title",
	search: "",
	sortDescending: false,
	filter: {
		architectures: {
			known: {
				X64: { enabled: false, locked: false },
				X86: { enabled: false, locked: false },
			},
			unknown: null,
		},
		engines: {
			known: {
				Unity: { enabled: false, locked: false },
				Unreal: { enabled: false, locked: false },
				Godot: { enabled: false, locked: false },
				GameMaker: { enabled: false, locked: false },
			},
			unknown: null,
		},
		providers: {
			known: {
				Epic: { enabled: false, locked: false },
				Gog: { enabled: false, locked: false },
				Itch: { enabled: false, locked: false },
				Manual: { enabled: false, locked: false },
				Steam: { enabled: false, locked: false },
				Xbox: { enabled: false, locked: false },
			},
			unknown: null,
		},
		tags: {
			known: {
				VR: { enabled: false, locked: false },
				Demo: { enabled: false, locked: false },
			},
			unknown: null,
		},
		unityBackends: {
			known: {
				Il2Cpp: { enabled: false, locked: false },
				Mono: { enabled: false, locked: false },
			},
			unknown: null,
		},
		installed: {
			known: {
				Installed: { enabled: false, locked: false },
				NotInstalled: { enabled: false, locked: false },
			},
			unknown: null,
		},
		modFamilies: { known: {}, unknown: null },
	},
};

export function useDataQuery() {
	const [query, setQuery] = useAppSettingSingle("gamesQuery");
	const queryRef = useRef(query || defaultQuery);

	useEffect(() => {
		queryRef.current = query || defaultQuery;
	}, [query]);

	const setQueryPartial = (partialQuery: Partial<GamesQuery> | null) => {
		const newQuery = partialQuery
			? { ...queryRef.current, ...partialQuery }
			: defaultQuery;
		setQuery(newQuery);
	};

	return [query || defaultQuery, setQueryPartial] as const;
}
