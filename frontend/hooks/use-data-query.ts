import { GamesQuery } from "@api/bindings";
import { useAppSettingSingle } from "./use-app-setting-single";
import { useEffect, useRef } from "react";
import { defaultQuery } from "./default-settings";

export function useDataQuery() {
	const [query, setQuery] = useAppSettingSingle("gamesQuery");
	const queryRef = useRef(query);

	useEffect(() => {
		queryRef.current = query;
	}, [query]);

	const setQueryPartial = (partialQuery: Partial<GamesQuery> | null) => {
		const newQuery = partialQuery
			? { ...queryRef.current, ...partialQuery }
			: defaultQuery;
		setQuery(newQuery);
	};

	return [query, setQueryPartial] as const;
}
