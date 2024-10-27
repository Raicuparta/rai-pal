import { useCallback } from "react";
import { usePersistedState } from "./use-persisted-state";

export type TableSort = {
	id?: string;
	reverse: boolean;
};

export function useTableSort(tableId: string, defaultId?: string) {
	const [sort, setSort] = usePersistedState<TableSort>(
		{
			id: defaultId,
			reverse: false,
		},
		`table-sort-${tableId}`,
	);

	const updateSort = useCallback(
		(id: string) => {
			setSort((previousSort) => ({
				id,
				reverse: previousSort?.id === id && !previousSort.reverse,
			}));
		},
		[setSort],
	);

	return [sort, updateSort] as const;
}
