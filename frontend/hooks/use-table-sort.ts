import { useCallback, useState } from "react";

export type TableSort = {
	id?: string;
	reverse: boolean;
};

export function useTableSort(defaultId?: string) {
	const [sort, setSort] = useState<TableSort>({
		id: defaultId,
		reverse: false,
	});

	const updateSort = useCallback((id: string) => {
		setSort((previousSort) => ({
			id,
			reverse: previousSort?.id === id && !previousSort.reverse,
		}));
	}, []);

	return [sort, updateSort] as const;
}
