import { GamesQuery } from "@api/bindings";
import { useAppSettingSingle } from "./use-app-setting-single";
import { useEffect, useRef } from "react";

export const defaultQuery: GamesQuery = {
	sortBy: "Title",
	search: "",
	sortDescending: false,
	filter: {
		architectures: [],
		engines: [],
		providers: [],
		tags: [],
		unityBackends: [],
		installed: [],
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
