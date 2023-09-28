import { useCallback, useState } from "react";

export type TableSort<TItem, TKey extends keyof TItem> = {
	id: TKey;
	reverse: boolean;
};

export function useTableSort<TItem, TKey extends keyof TItem>(defaultId: TKey) {
	const [sort, setSort] = useState<TableSort<TItem, TKey>>({
		id: defaultId,
		reverse: false,
	});

	const updateSort = useCallback((id: TKey) => {
		setSort((previousSort) => ({
			id,
			reverse: previousSort?.id === id && !previousSort.reverse,
		}));
	}, []);

	return [sort, updateSort] as const;
}
