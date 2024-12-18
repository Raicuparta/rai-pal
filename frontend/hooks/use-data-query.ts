import { type Result, type Error } from "@api/bindings";
import { useCallback, useEffect, useState } from "react";

export function useDataQuery<TQuery>(
	setterCommand: (filter: TQuery | null) => Promise<Result<null, Error>>,
	getterCommand: () => Promise<Result<TQuery, Error>>,
) {
	const [query, setQuery] = useState<TQuery | null>(null);

	const updateFilters = useCallback(() => {
		getterCommand().then((result) => {
			if (result.status === "ok") {
				setQuery(result.data);
			}
		});
	}, [getterCommand]);

	useEffect(() => {
		updateFilters();
	}, [updateFilters]);

	const setQueryExternal = useCallback(
		(partialQuery: Partial<TQuery>) => {
			setQuery((previousQuery) => {
				if (!previousQuery) {
					console.warn("Tried to set data query before it was ready.");
					return previousQuery;
				}

				const newQuery = {
					...previousQuery,
					...partialQuery,
				};

				setterCommand(newQuery);

				return newQuery;
			});
		},
		[setterCommand],
	);

	return [query, setQueryExternal] as const;
}
