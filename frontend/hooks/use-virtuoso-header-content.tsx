import { TableColumn, TableHead } from "@components/table/table-head";
import { useCallback } from "react";
import { GamesSortBy } from "@api/bindings";

export function useVirtuosoHeaderContent<TKey extends string, TItem, TSort>(
	columns: TableColumn<TKey, TItem, TSort>[],
	onChangeSort?: (sort: TSort) => void,
	sort?: GamesSortBy,
	sortDescending?: boolean,
) {
	return useCallback(
		() => (
			<TableHead
				columns={columns}
				onChangeSort={onChangeSort}
				sortBy={sort}
				sortDescending={sortDescending}
			/>
		),
		[columns, onChangeSort, sort, sortDescending],
	);
}
